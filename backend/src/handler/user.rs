use crate::{
    api::user::{
        EmailVerificationInput, LogInUserInput, LogOutUserResponse, SessionToken, SignUpUserInput,
        User, UserAuthProvider::Password,
    },
    error::app_error::{
        internal_error, AppError, FailedValidation,
        ValidationIssue::{Invalid, Required, TooWeak},
    },
    json::extractor::Extractor,
    storage::error::ErrorCode,
    AppState,
};

use axum::{
    body::Body,
    extract::{Query, State},
    http::{header::AUTHORIZATION, Request},
    Extension,
};
use chrono::Local;
use email_address::EmailAddress;

/// Creates a new user in the system.
pub async fn sign_up_user(
    State(state): State<AppState>,
    Extractor(input): Extractor<SignUpUserInput>,
) -> Result<Extractor<User>, AppError> {
    let mut validation_errors = vec![];
    if state.user_helper.password_is_weak(&input.password) {
        validation_errors.push(FailedValidation {
            field: "password".to_string(),
            issue: TooWeak,
        });
    }

    if input.name.trim().len() == 0 {
        validation_errors.push(FailedValidation {
            field: "name".to_string(),
            issue: Required,
        });
    }

    if state
        .user_helper
        .is_bot(&state.settings.captcha.secret, &input.captcha)
        .await
    {
        validation_errors.push(FailedValidation {
            field: "captcha".to_string(),
            issue: Invalid,
        });
    }

    let clean_mail = input.email.trim().to_lowercase();
    if !EmailAddress::is_valid(&clean_mail) {
        validation_errors.push(FailedValidation {
            field: "email".to_string(),
            issue: Invalid,
        });
    }

    if !validation_errors.is_empty() {
        return Err(AppError::ValidationError(validation_errors));
    }

    let password_hash = match state.user_helper.hash(&input.password) {
        Ok(h) => h,
        Err(e) => return Err(e),
    };

    let id = format!("{}+{}", clean_mail, Password);
    let user = User {
        id,
        provider: Password,
        email: clean_mail,
        name: input.name.clone(),
        password: Some(password_hash),
        email_verified_at: None,
        recorded_at: Local::now(),
    };
    let inserted = state
        .storage
        .sign_up_user(
            user.clone(),
            state.settings.clone(),
            state.mail_helper.clone(),
        )
        .await?;

    Ok(Extractor(User {
        id: inserted.id,
        provider: inserted.provider,
        email: inserted.email,
        name: inserted.name,
        password: None,
        email_verified_at: None,
        recorded_at: inserted.recorded_at,
    }))
}

/// Validates new user's e-mail address
pub async fn email_verification(
    State(state): State<AppState>,
    Query(input): Query<EmailVerificationInput>,
) -> Result<Extractor<User>, AppError> {
    if input.token.trim().len() == 0 {
        let validation_errors = vec![FailedValidation {
            field: "token".to_string(),
            issue: Required,
        }];
        return Err(AppError::ValidationError(validation_errors));
    }

    match state.storage.verify_email(&input.token).await {
        Ok(u) => {
            return Ok(Extractor(User {
                id: u.id,
                provider: u.provider,
                email: u.email,
                name: u.name,
                password: None,
                email_verified_at: u.email_verified_at,
                recorded_at: u.recorded_at,
            }));
        }
        Err(e) => {
            if e.code == ErrorCode::NotFoundError {
                let validation_errors = vec![FailedValidation {
                    field: "token".to_string(),
                    issue: Invalid,
                }];
                return Err(AppError::ValidationError(validation_errors));
            }
            return internal_error();
        }
    }
}

/// Logs in a user
pub async fn log_in_user(
    State(state): State<AppState>,
    Extractor(input): Extractor<LogInUserInput>,
) -> Result<Extractor<SessionToken>, AppError> {
    let validation_errors = vec![FailedValidation {
        field: "email".to_string(),
        issue: Invalid,
    }];

    let clean_mail = input.email.trim().to_lowercase();
    if !EmailAddress::is_valid(&clean_mail) {
        return Err(AppError::ValidationError(validation_errors));
    }

    let id = format!("{}+{}", clean_mail, Password);
    let user = match state.storage.user_by_id(&id).await {
        Ok(u) => {
            if u.is_none() {
                return Err(AppError::ValidationError(validation_errors));
            }

            u.unwrap()
        }
        Err(_) => {
            return Err(AppError::ServerError);
        }
    };

    if user.email_verified_at.is_none() {
        return Err(AppError::ValidationError(validation_errors));
    }

    let valid = match state
        .user_helper
        .verify_hash(&input.password, &user.password.unwrap())
    {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    if !valid {
        return Err(AppError::ValidationError(validation_errors));
    }

    let session_token = match state.storage.log_in(&id).await {
        Ok(t) => t,
        Err(_) => {
            return Err(AppError::ServerError);
        }
    };

    return Ok(Extractor(session_token));
}

/// Returns currently logged in user
pub async fn me(user: Extension<User>) -> Result<Extractor<User>, AppError> {
    Ok(Extractor((*user).clone()))
}

/// Deletes the user session in the request
pub async fn log_out_user(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Extractor<LogOutUserResponse>, AppError> {
    let auth_header = match request.headers().get(AUTHORIZATION) {
        Some(h) => h.to_str().unwrap(),
        None => return internal_error(),
    };
    match state.storage.log_out(auth_header).await {
        Ok(_) => Ok(Extractor(LogOutUserResponse {})),
        Err(_) => return internal_error(),
    }
}
