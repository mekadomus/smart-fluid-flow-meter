use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct SaveMeasurementInput {
    pub device_id: String,
    pub measurement: String,
}

#[derive(Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Measurement {
    pub id: Option<String>,
    pub device_id: String,
    pub measurement: String,
    pub recorded_at: NaiveDateTime,
}
