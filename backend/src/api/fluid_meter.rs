use crate::api::common::SortDirection;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum FluidMeterStatus {
    Active,
    // Still shown to the user, but not triggering alarms
    Inactive,
    // Not showen to the user
    Deleted,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FluidMetersSort {
    Id,
    Name,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FluidMeter {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub status: FluidMeterStatus,
    pub recorded_at: DateTime<Local>,
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
