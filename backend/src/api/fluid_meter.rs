use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::api::common::SortDirection;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")] // Store as a string in the DB
pub enum FluidMeterStatus {
    #[sqlx(rename = "active")]
    Active,
    // Still shown to the user, but not triggering alarms
    #[sqlx(rename = "inactive")]
    Inactive,
    // Not shown to the user
    #[sqlx(rename = "deleted")]
    Deleted,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FluidMetersSort {
    Id,
    Name,
}

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct FluidMeter {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub status: FluidMeterStatus,
    pub recorded_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FluidMetersInput {
    pub sort: Option<FluidMetersSort>,
    pub sort_direction: Option<SortDirection>,
    // Will retrieve only items after this one
    pub page_cursor: Option<String>,
    pub page_size: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateFluidMeterInput {
    pub name: String,
}
