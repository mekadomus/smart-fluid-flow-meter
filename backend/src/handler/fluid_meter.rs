use crate::{
    api::{
        common::{PaginatedResponse, Pagination, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE},
        fluid_meter::{FluidMeter, FluidMetersInput},
        user::User,
    },
    error::app_error::{internal_error, AppError, FailedValidation, ValidationIssue::TooLarge},
    json::extractor::Extractor,
    AppState,
};

use axum::{
    body::Body,
    extract::{Query, State},
    http::Request,
};

/// Lists all fluid meters for the logged in user
pub async fn fluid_meters(
    State(state): State<AppState>,
    Query(input): Query<FluidMetersInput>,
    request: Request<Body>,
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

    let user = match request.extensions().get::<User>() {
        Some(u) => u,
        None => return internal_error(),
    };

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
