use chrono::NaiveDateTime;
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

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")] // Store as a string in the DB
pub enum UserAuthProvider {
    #[serde(rename = "password")]
    #[sqlx(rename = "password")]
    Password,
}

impl fmt::Display for UserAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserAuthProvider::Password => write!(f, "password"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub provider: UserAuthProvider,
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub email_verified_at: Option<NaiveDateTime>,
    pub recorded_at: NaiveDateTime,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SessionToken {
    pub user_id: String,
    pub token: String,
    pub expires_at: NaiveDateTime,
}
