use axum::{
    extract::{Query, State},
    Extension,
};
use chrono::Local;
use uuid::Uuid;

use crate::{
    api::{
        common::{PaginatedResponse, Pagination, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE},
        fluid_meter::{CreateFluidMeterInput, FluidMeter, FluidMeterStatus, FluidMetersInput},
        user::User,
    },
    error::app_error::{
        internal_error, AppError, FailedValidation,
        ValidationIssue::{Required, TooLarge},
    },
    json::extractor::Extractor,
    AppState,
};

/// Lists all fluid meters for the logged in user
pub async fn fluid_meters(
    State(state): State<AppState>,
    Query(input): Query<FluidMetersInput>,
    user: Extension<User>,
) -> Result<Extractor<PaginatedResponse<FluidMeter>>, AppError> {
    let mut validation_errors = vec![];

    let page_size = input.page_size.unwrap_or(*DEFAULT_PAGE_SIZE);
    if page_size > *MAX_PAGE_SIZE {
        validation_errors.push(FailedValidation {
            field: "page_size".to_string(),
            issue: TooLarge,
        });
    }

    if !validation_errors.is_empty() {
        return Err(AppError::ValidationError(validation_errors));
    }

    let mut options = input.clone();
    options.page_size = Some(page_size + 1);

    let mut meters = match state.storage.get_fluid_meters(&user.id, &options).await {
        Ok(m) => m,
        Err(_) => return internal_error(),
    };
    let has_more = (meters.len() as u8) > page_size;
    if has_more {
        meters.pop();
    }

    let resp = PaginatedResponse {
        items: meters,
        pagination: Pagination {
            has_more,
            has_less: input.page_cursor.is_some(),
        },
    };

    return Ok(Extractor(resp));
}

/// Creates a new fluid meter
pub async fn create_fluid_meter(
    State(state): State<AppState>,
    user: Extension<User>,
    Extractor(input): Extractor<CreateFluidMeterInput>,
) -> Result<Extractor<FluidMeter>, AppError> {
    let mut validation_errors = vec![];

    let name = input.name.trim();
    if name.len() == 0 {
        validation_errors.push(FailedValidation {
            field: "name".to_string(),
            issue: Required,
        });
    }

    if !validation_errors.is_empty() {
        return Err(AppError::ValidationError(validation_errors));
    }

    let meter = FluidMeter {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        status: FluidMeterStatus::Active,
        owner_id: user.id.clone(),
        recorded_at: Local::now(),
    };

    let meter = match state.storage.insert_fluid_meter(&meter).await {
        Ok(m) => m,
        Err(_) => return internal_error(),
    };

    return Ok(Extractor(meter));
}
