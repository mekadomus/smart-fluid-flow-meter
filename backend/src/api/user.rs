use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Deserialize, Serialize)]
pub struct SignUpUserInput {
    pub captcha: String,
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LogInUserInput {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LogOutUserResponse {}

#[derive(Clone, Deserialize, Serialize)]
pub struct EmailVerificationInput {
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum UserAuthProvider {
    #[serde(rename = "password")]
    Password,
}

impl fmt::Display for UserAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserAuthProvider::Password => write!(f, "password"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub provider: UserAuthProvider,
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub email_verified_at: Option<DateTime<Local>>,
    pub recorded_at: DateTime<Local>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SessionToken {
    pub user_id: String,
    pub token: String,
    pub expiration: DateTime<Local>,
}
