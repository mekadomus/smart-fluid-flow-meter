use crate::{
    error::app_error::AppError,
    http_client::cloudflare::{check_captcha, CheckCaptchaRequest},
    storage::Storage,
};

use async_trait::async_trait;
use mockall::automock;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;
use zxcvbn::{zxcvbn, Score::Three};

#[automock]
#[async_trait]
pub trait UserHelper: Send + Sync {
    async fn is_bot(&self, secret: &str, token: &str) -> bool;
    fn password_is_weak(&self, pass: &str) -> bool;
    fn hash(&self, input: &str) -> Result<String, AppError>;
    fn verify_hash(&self, input: &str, hash: &str) -> Result<bool, AppError>;
    async fn owns_fluid_meter(
        &self,
        storage: Arc<dyn Storage>,
        user_id: &str,
        meter_id: &str,
    ) -> Result<bool, AppError>;
}

pub struct DefaultUserHelper;

#[async_trait]
impl UserHelper for DefaultUserHelper {
    /// Returns true if the given password is too easy to guess, so we can't accept it
    fn password_is_weak(&self, pass: &str) -> bool {
        let strength = zxcvbn(pass, &[]);
        return strength.score() < Three;
    }

    /// Checks if the user is a bot by verifying a captcha token
    ///
    /// # Arguments
    /// * `secret` - Cloudflare turnstile secret
    /// * `token` - Captcha token provided by the client
    async fn is_bot(&self, secret: &str, token: &str) -> bool {
        let req = CheckCaptchaRequest {
            secret,
            response: token,
        };
        match check_captcha(req).await {
            Ok(r) => {
                return !r;
            }
            Err(e) => {
                error!("Error checking if user is bot. {}", e);
                return true;
            }
        }
    }

    /// Given a string (usually a password), it generates a hash with salt
    fn hash(&self, input: &str) -> Result<String, AppError> {
        return match bcrypt::hash(input, bcrypt::DEFAULT_COST) {
            Ok(h) => Ok(h),
            Err(e) => {
                error!("Failed to hash password. {}", e);
                Err(AppError::ServerError)
            }
        };
    }

    /// Given a string (usually a password), and a hashed password. It verifies
    /// if they match
    fn verify_hash(&self, input: &str, hash: &str) -> Result<bool, AppError> {
        match bcrypt::verify(input, hash) {
            Ok(v) => return Ok(v),
            Err(_) => {
                return Err(AppError::ServerError);
            }
        };
    }

    async fn owns_fluid_meter(
        &self,
        storage: Arc<dyn Storage>,
        user_id: &str,
        meter_id: &str,
    ) -> Result<bool, AppError> {
        return Ok(!Uuid::try_parse(&meter_id).is_err()
            && storage.is_fluid_meter_owner(&meter_id, &user_id).await?);
    }
}

#[cfg(test)]
mod tests {
    use super::{DefaultUserHelper, UserHelper};
    use zxcvbn::{
        zxcvbn,
        Score::{Three, Two},
    };

    #[test]
    fn password_is_weak_rejects_score_2() {
        let password = "hello.2?a";
        assert_eq!(zxcvbn(password, &[]).score(), Two);

        let helper = DefaultUserHelper {};
        assert!(helper.password_is_weak(password));
    }

    #[test]
    fn password_is_weak_accepts_score_3() {
        let password = "Muchos.tacos";
        assert_eq!(zxcvbn(password, &[]).score(), Three);

        let helper = DefaultUserHelper {};
        assert!(!helper.password_is_weak(password))
    }

    #[test]
    fn hash_can_be_verified() {
        let password = "Muchos.tacos";
        let helper = DefaultUserHelper {};
        let hash = match helper.hash(password) {
            Ok(h) => h,
            Err(_) => {
                panic!("Wasn't able to hash password");
            }
        };

        match bcrypt::verify(password, &hash) {
            Ok(r) => {
                assert!(r);
            }
            Err(_) => {
                panic!("Hash couldn't be verified");
            }
        }
    }

    #[test]
    fn hash_fails_with_incorrect_password() {
        let password = "Muchos.tacos";
        let helper = DefaultUserHelper {};
        let hash = match helper.hash(password) {
            Ok(h) => h,
            Err(_) => {
                panic!("Wasn't able to hash password");
            }
        };

        match bcrypt::verify("Incorrect.password", &hash) {
            Ok(r) => {
                assert!(!r);
            }
            Err(_) => {
                panic!("Hash couldn't be verified");
            }
        }
    }
}
