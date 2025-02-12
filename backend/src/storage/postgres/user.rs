use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::error;

use crate::{
    api::{
        email_verification::EmailVerification,
        user::{SessionToken, User},
    },
    helper::{
        mail::MailHelper,
        token::{alphanumeric, AUTH_TOKEN_LEN},
    },
    settings::settings::Settings,
    storage::{
        error::{not_found, undefined, Error},
        postgres::PostgresStorage,
        UserStorage,
    },
};

#[async_trait]
impl UserStorage for PostgresStorage {
    /// Used in tests. Production users should use sign_up_user
    async fn insert_user(&self, user: &User) -> Result<User, Error> {
        match sqlx::query("INSERT INTO account(id, provider, email, password, name, email_verified_at, recorded_at) VALUES($1, $2, $3, $4, $5, $6, $7)")
        .bind(&user.id)
        .bind(&user.provider)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.name)
        .bind(&user.email_verified_at)
        .bind(&user.recorded_at)
        .execute(&self.pool)
        .await
        {
            Ok(_) => {},
            Err(err) => {
                error!("Error inserting user: {}", err);
                return undefined();
            }
        };

        return Ok(user.clone());
    }

    /// Saves a new user to the DB, creates an e-mail verification token and sends
    /// an e-mail for verification. The user needs to verify their e-mail before
    /// they can use their accout
    async fn sign_up_user(
        &self,
        user: User,
        settings: Arc<Settings>,
        mail_helper: Arc<dyn MailHelper>,
    ) -> Result<User, Error> {
        let mut tx = match self.pool.begin().await {
            Ok(t) => t,
            Err(e) => {
                error!("Error inserting user. {}", e);
                return undefined();
            }
        };

        match sqlx::query("INSERT INTO account(id, provider, email, password, name, recorded_at) VALUES($1, $2, $3, $4, $5, $6)")
        .bind(&user.id)
        .bind(&user.provider)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.name)
        .bind(user.recorded_at)
        .execute(&mut *tx)
        .await
        {
            Ok(_) => {},
            Err(err) => {
                error!("Error inserting user: {}", err);
                return undefined();
            }
        };

        let verification_token = alphanumeric(&100);
        match sqlx::query(
            "INSERT INTO email_verification(token, account_id, recorded_at) VALUES($1, $2, $3)",
        )
        .bind(&verification_token)
        .bind(&user.id)
        .bind(user.recorded_at)
        .execute(&mut *tx)
        .await
        {
            Ok(_) => {
                if !mail_helper
                    .sign_up_verification(&settings, &user, &verification_token)
                    .await
                {
                    let _ = tx.rollback().await;
                    error!("Couldn't send verification email");
                    return undefined();
                }
            }
            Err(e) => {
                error!("Error saving verification: {}", e);
                return undefined();
            }
        };

        let _ = tx.commit().await;
        return Ok(user);
    }

    async fn user_by_id(&self, id: &str) -> Result<Option<User>, Error> {
        match sqlx::query_as("SELECT * FROM account WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(f) => return Ok(Some(f)),
            Err(e) => {
                match e {
                    sqlx::Error::RowNotFound => {
                        return Ok(None);
                    }
                    _ => {}
                }

                error!("Error getting user by id: {}", e);
                return undefined();
            }
        };
    }

    async fn user_by_token(&self, token: &str) -> Result<Option<User>, Error> {
        match sqlx::query_as(
            r#"
                SELECT
                    account.*
                FROM
                    session_token
                LEFT JOIN
                    account ON account.id = session_token.account_id
                WHERE
                    session_token.token = $1 AND session_token.expires_at > $2
                "#,
        )
        .bind(token)
        .bind(Utc::now().naive_utc())
        .fetch_one(&self.pool)
        .await
        {
            Ok(u) => return Ok(Some(u)),
            Err(e) => {
                match e {
                    sqlx::Error::RowNotFound => {
                        return Ok(None);
                    }
                    _ => {}
                }

                error!("Failed to query for session token: {}", e);
                return undefined();
            }
        };
    }

    async fn verify_email(&self, token: &str) -> Result<User, Error> {
        let mut tx = match self.pool.begin().await {
            Ok(t) => t,
            Err(e) => {
                error!("Error creating transaction. {}", e);
                return undefined();
            }
        };

        let mut user: User = match sqlx::query_as(
            r#"
                SELECT
                    account.*
                FROM
                    email_verification
                LEFT JOIN
                    account ON account.id = email_verification.account_id
                WHERE
                    token = $1
                "#,
        )
        .bind(token)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(u) => u,
            Err(e) => {
                match e {
                    sqlx::Error::RowNotFound => {
                        error!("No verification matches token {}", &token);
                        return not_found();
                    }
                    _ => {}
                }

                error!("Failed to query for email_verification {}", e);
                return undefined();
            }
        };

        user.email_verified_at = Some(Utc::now().naive_utc());

        match sqlx::query("UPDATE account SET email_verified_at = $1 WHERE id = $2")
            .bind(&user.email_verified_at)
            .bind(&user.id)
            .execute(&mut *tx)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                let _ = tx.rollback().await;
                error!("Failed to update user. {}", e);
                return undefined();
            }
        };

        let _ = tx.commit().await;
        return Ok(user);
    }

    async fn email_verification_by_id(&self, id: &str) -> Result<Option<EmailVerification>, Error> {
        match sqlx::query_as("SELECT * FROM email_verification WHERE account_id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(f) => return Ok(Some(f)),
            Err(e) => {
                match e {
                    sqlx::Error::RowNotFound => {
                        return Ok(None);
                    }
                    _ => {}
                }

                error!("Error in email_verification_by_id: {}", e);
                return undefined();
            }
        };
    }

    async fn log_in(&self, id: &str) -> Result<SessionToken, Error> {
        let token = SessionToken {
            user_id: id.to_string(),
            token: alphanumeric(AUTH_TOKEN_LEN),
            expires_at: Utc::now().naive_utc() + Duration::days(30),
        };

        match sqlx::query(
            "INSERT INTO session_token(token, account_id, expires_at) VALUES($1, $2, $3)",
        )
        .bind(&token.token)
        .bind(&token.user_id)
        .bind(&token.expires_at)
        .execute(&self.pool)
        .await
        {
            Ok(_) => return Ok(token),
            Err(e) => {
                error!("Error logging in: {}", e);
                return undefined();
            }
        };
    }

    async fn log_out(&self, token: &str) -> Result<(), Error> {
        match sqlx::query("DELETE FROM session_token WHERE token = $1")
            .bind(&token)
            .execute(&self.pool)
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Error deleting session: {}", e);
                return undefined();
            }
        };
    }
}
