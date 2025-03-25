use crate::api::{alert::Alert, common::SortDirection};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for FluidMeterStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FluidMeterStatus::Active => write!(f, "active"),
            FluidMeterStatus::Inactive => write!(f, "inactive"),
            FluidMeterStatus::Deleted => write!(f, "deleted"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FluidMetersSort {
    Id,
    Name,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, sqlx::FromRow)]
pub struct FluidMeter {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub status: FluidMeterStatus,
    pub recorded_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FluidMetersInput {
    pub sort: Option<FluidMetersSort>,
    pub sort_direction: Option<SortDirection>,
    pub status: Option<FluidMeterStatus>,
    // Will retrieve only items after this one
    pub page_cursor: Option<String>,
    pub page_size: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateFluidMeterInput {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FluidMeterAlerts {
    pub meter: FluidMeter,
    pub alerts: Vec<Alert>,
}
