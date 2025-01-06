use crate::{
    api::user::{SignUpUserInput, User, UserAuthProvider::Password},
    error::app_error::{
        AppError, FailedValidation,
        ValidationIssue::{Invalid, Required, TooWeak},
    },
    json::extractor::Extractor,
    AppState,
};

use axum::extract::State;
use chrono::Local;
use email_address::EmailAddress;

/**
 * Creates a new user in the system.
 */
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
        .is_bot(&state.settings.captcha.secret, &input.captcha, "userip")
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
