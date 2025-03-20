use axum::{
    extract::{Path, Query, State},
    Extension,
};
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

use crate::{
    api::{
        common::{PaginatedResponse, Pagination, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE},
        fluid_meter::{CreateFluidMeterInput, FluidMeter, FluidMeterStatus, FluidMetersInput},
        user::User,
    },
    error::app_error::{
        internal_error, AppError, FailedValidation,
        ValidationIssue::{Invalid, Required, TooLarge},
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
        recorded_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    let meter = match state.storage.insert_fluid_meter(&meter).await {
        Ok(m) => m,
        Err(_) => return internal_error(),
    };

    return Ok(Extractor(meter));
}

/// Gets information about a specific fluid meter
pub async fn get_fluid_meter(
    State(state): State<AppState>,
    Path(meter_id): Path<String>,
    user: Extension<User>,
) -> Result<Extractor<FluidMeter>, AppError> {
    if !state
        .user_helper
        .owns_fluid_meter(state.storage.clone(), &user.id, &meter_id)
        .await?
    {
        return Err(AppError::ValidationError(vec![FailedValidation {
            field: "meter_id".to_string(),
            issue: Invalid,
        }]));
    }

    match state.storage.get_fluid_meter_by_id(&meter_id).await {
        Ok(m) => {
            if m.is_none() {
                error!("User owns none existing meter: {}", meter_id);
                return internal_error();
            }

            return Ok(Extractor(m.unwrap()));
        }
        Err(e) => {
            error!("Error getting fluid meter: {}. Error: {}", meter_id, e);
            return internal_error();
        }
    }
}

/// Activate given meter
pub async fn activate_fluid_meter(
    State(state): State<AppState>,
    Path(meter_id): Path<String>,
    user: Extension<User>,
) -> Result<Extractor<()>, AppError> {
    if !state
        .user_helper
        .owns_fluid_meter(state.storage.clone(), &user.id, &meter_id)
        .await?
    {
        return Err(AppError::ValidationError(vec![FailedValidation {
            field: "meter_id".to_string(),
            issue: Invalid,
        }]));
    }

    match state.storage.activate_fluid_meter(&meter_id).await {
        Ok(_) => {
            return Ok(Extractor(()));
        }
        Err(e) => {
            error!("Error getting fluid meter: {}. Error: {}", meter_id, e);
            return internal_error();
        }
    }
}

/// Deactivate given meter
pub async fn deactivate_fluid_meter(
    State(state): State<AppState>,
    Path(meter_id): Path<String>,
    user: Extension<User>,
) -> Result<Extractor<()>, AppError> {
    if !state
        .user_helper
        .owns_fluid_meter(state.storage.clone(), &user.id, &meter_id)
        .await?
    {
        return Err(AppError::ValidationError(vec![FailedValidation {
            field: "meter_id".to_string(),
            issue: Invalid,
        }]));
    }

    match state.storage.deactivate_fluid_meter(&meter_id).await {
        Ok(_) => {
            return Ok(Extractor(()));
        }
        Err(e) => {
            error!("Error getting fluid meter: {}. Error: {}", meter_id, e);
            return internal_error();
        }
    }
}
