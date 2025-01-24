use async_trait::async_trait;
use chrono::{Duration, Local};
use firestore::{path, paths, FirestoreResult};
use futures::{stream::BoxStream, TryStreamExt};
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
        error::{not_found, undefined, Error, ErrorCode},
        firestore::FirestoreStorage,
        UserStorage,
    },
};

const EMAIL_VERIFICATION_COLLECTION: &'static str = "email_verification";
const SESSION_TOKEN_COLLECTION: &'static str = "session_token";
const USER_COLLECTION: &'static str = "user";

#[async_trait]
impl UserStorage for FirestoreStorage {
    /// Saves a new user to the DB, creates an e-mail verification token and sends
    /// an e-mail for verification. The user needs to verify their e-mail before
    /// they can use their accout
    async fn sign_up_user(
        &self,
        user: User,
        settings: Arc<Settings>,
        mail_helper: Arc<dyn MailHelper>,
    ) -> Result<User, Error> {
        // This should be in a transaction, but firestore doesn't really allow to
        // do this transactionally, so it doesn't really work as a transaction.
        // Leaving it like this in case we change to a storage system that allows
        // transactions
        let inserted: User = match self
            .db
            .fluent()
            .insert()
            .into(USER_COLLECTION)
            .document_id(&user.id)
            .object(&user)
            .execute()
            .await
        {
            Ok(inserted) => inserted,
            Err(e) => {
                error!("Error inserting user. {}", e);
                return undefined();
            }
        };

        let verification_token = alphanumeric(&100);
        let verification = EmailVerification {
            id: inserted.id.clone(),
            token: verification_token,
            recorded_at: Local::now(),
        };
        match self
            .db
            .fluent()
            .insert()
            .into(EMAIL_VERIFICATION_COLLECTION)
            .document_id(&verification.id)
            .object(&verification)
            .execute::<EmailVerification>()
            .await
        {
            Ok(_) => {
                if !mail_helper
                    .sign_up_verification(&settings, &user, &verification.token)
                    .await
                {
                    error!("Couldn't send verification email");
                    return undefined();
                }
            }
            Err(e) => {
                error!("Error saving verification: {}", e);
                return undefined();
            }
        };

        return Ok(inserted);
    }

    async fn user_by_id(&self, id: &str) -> Result<Option<User>, Error> {
        match self
            .db
            .fluent()
            .select()
            .by_id_in(USER_COLLECTION)
            .obj()
            .one(id)
            .await
        {
            Ok(f) => return Ok(f),
            Err(err) => {
                error!("Error finding user: {}", err);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        };
    }

    async fn user_by_token(&self, token: &str) -> Result<Option<User>, Error> {
        let res: BoxStream<FirestoreResult<SessionToken>> = match self
            .db
            .fluent()
            .select()
            .from(SESSION_TOKEN_COLLECTION)
            .filter(|q| q.field(path!(SessionToken::token)).eq(token))
            .obj()
            .stream_query_with_errors()
            .await
        {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to query for session token: {}", e);
                return undefined();
            }
        };

        let sessions: Vec<SessionToken> = match res.try_collect().await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to query for session token: {}", e);
                return undefined();
            }
        };

        if sessions.is_empty() {
            return Ok(None);
        }

        let user_opt: Option<User> = match self
            .db
            .fluent()
            .select()
            .by_id_in(USER_COLLECTION)
            .obj()
            .one(&sessions[0].user_id)
            .await
        {
            Ok(u) => u,
            Err(e) => {
                error!("Error retrieving user {}. Error {}", sessions[0].user_id, e);
                return undefined();
            }
        };

        if user_opt.is_none() {
            error!(
                "User {} not found. Even when it has a session",
                sessions[0].user_id
            );
            return undefined();
        }

        return Ok(Some(user_opt.unwrap()));
    }

    async fn verify_email(&self, token: &str) -> Result<User, Error> {
        let tkn = token.to_string();
        let res: BoxStream<FirestoreResult<EmailVerification>> = match self
            .db
            .fluent()
            .select()
            .from(EMAIL_VERIFICATION_COLLECTION)
            .filter(|q| q.field(path!(EmailVerification::token)).eq(tkn.clone()))
            .obj()
            .stream_query_with_errors()
            .await
        {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to query for email verification: {}", e);
                return undefined();
            }
        };

        let verifications: Vec<EmailVerification> = match res.try_collect().await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to query for email verification: {}", e);
                return undefined();
            }
        };

        if verifications.is_empty() {
            error!("No verification matches token {}", tkn);
            return not_found();
        }

        let user_opt: Option<User> = match self
            .db
            .fluent()
            .select()
            .by_id_in(USER_COLLECTION)
            .obj()
            .one(&verifications[0].id)
            .await
        {
            Ok(u) => u,
            Err(e) => {
                error!("Error retrieving user {}. Error {}", verifications[0].id, e);
                return undefined();
            }
        };

        if user_opt.is_none() {
            error!("User {} not found", verifications[0].id);
            return undefined();
        }
        let mut user = user_opt.unwrap();
        user.email_verified_at = Some(Local::now());

        let updated = self
            .db
            .fluent()
            .update()
            .fields(paths!(User::email_verified_at))
            .in_col(USER_COLLECTION)
            .document_id(&verifications[0].id)
            .object(&user)
            .execute::<User>()
            .await;

        if updated.is_err() {
            error!(
                "Failed to update user {}. Error: {}",
                user.id,
                updated.err().unwrap()
            );
            return undefined();
        }

        return Ok(updated.unwrap());
    }

    async fn email_verification_by_id(&self, id: &str) -> Result<Option<EmailVerification>, Error> {
        match self
            .db
            .fluent()
            .select()
            .by_id_in(EMAIL_VERIFICATION_COLLECTION)
            .obj()
            .one(id)
            .await
        {
            Ok(f) => return Ok(f),
            Err(err) => {
                error!("Error finding email_verification: {}", err);
                return undefined();
            }
        };
    }

    async fn log_in(&self, id: &str) -> Result<SessionToken, Error> {
        let token = SessionToken {
            user_id: id.to_string(),
            token: alphanumeric(AUTH_TOKEN_LEN),
            expiration: Local::now() + Duration::days(30),
        };

        match self
            .db
            .fluent()
            .insert()
            .into(SESSION_TOKEN_COLLECTION)
            .document_id(&token.token)
            .object(&token)
            .execute()
            .await
        {
            Ok(t) => return Ok(t),
            Err(e) => {
                error!("Error creating session token. {}", e);
                return undefined();
            }
        };
    }

    async fn log_out(&self, token: &str) -> Result<(), Error> {
        match self
            .db
            .fluent()
            .delete()
            .from(SESSION_TOKEN_COLLECTION)
            .document_id(token)
            .execute()
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Error deleting session token. {}", e);
                return undefined();
            }
        };
    }
}
