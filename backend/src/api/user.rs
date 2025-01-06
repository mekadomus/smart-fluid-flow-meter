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

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub provider: UserAuthProvider,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub email_verified_at: Option<DateTime<Local>>,
    pub recorded_at: DateTime<Local>,
}
