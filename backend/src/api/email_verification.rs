use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct EmailVerification {
    pub id: String,
    pub token: String,
    pub recorded_at: DateTime<Local>,
}
