use crate::api::common::SeriesGranularity;

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct SaveMeasurementInput {
    pub device_id: String,
    pub measurement: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GetMeasurementsInput {
    pub granularity: Option<SeriesGranularity>,
    // If granularity is set to Hour, this should be set to a date. The response
    // will include the data points for each hour on that date
    pub day: Option<NaiveDate>,
}

#[derive(Clone, Deserialize, Serialize, sqlx::FromRow, Debug)]
pub struct Measurement {
    pub id: String,
    pub device_id: String,
    pub measurement: String,
    pub recorded_at: NaiveDateTime,
}
