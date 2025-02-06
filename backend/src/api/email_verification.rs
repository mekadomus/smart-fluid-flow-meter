use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct EmailVerification {
    pub token: String,
    pub account_id: String,
    pub recorded_at: NaiveDateTime,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct EmailVerificationInput {
    pub token: String,
}
