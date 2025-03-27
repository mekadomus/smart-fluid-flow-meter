use axum::{
    extract::{Path, Query, State},
    Extension,
};
use chrono::{Duration, NaiveDateTime, NaiveTime, Utc};
use uuid::Uuid;

use crate::{
    api::{
        common::{Series, SeriesGranularity, SeriesItem},
        fluid_meter::FluidMeterStatus::Active,
        measurement::{GetMeasurementsInput, Measurement, SaveMeasurementInput},
        user::User,
    },
    error::app_error::{
        internal_error, validation_error, AppError, FailedValidation,
        ValidationIssue::{Invalid, Required, TooFrequent},
    },
    helper::measurement::create_series,
    json::extractor::Extractor,
    storage::error::ErrorCode::RateLimitError,
    AppState,
};

pub async fn save_measurement(
    State(state): State<AppState>,
    Extractor(input): Extractor<SaveMeasurementInput>,
) -> Result<Extractor<Measurement>, AppError> {
    let meter = state
        .storage
        .get_fluid_meter_by_id(&input.device_id)
        .await?;
    match meter {
        Some(m) => {
            if m.status == Active {
                let measurement = Measurement {
                    id: Uuid::new_v4().to_string(),
                    device_id: input.device_id,
                    measurement: input.measurement,
                    recorded_at: Utc::now().naive_utc(),
                };
                match state.storage.save_measurement(&measurement).await {
                    Ok(_) => {}
                    Err(e) => {
                        if e.code == RateLimitError {
                            let validation_errors = vec![FailedValidation {
                                field: "request".to_string(),
                                issue: TooFrequent,
                            }];
                            return validation_error(validation_errors);
                        }
                        return internal_error();
                    }
                }

                return Ok(Extractor(measurement));
            }
        }
        None => {}
    }

    return validation_error(vec![FailedValidation {
        field: "device_id".to_string(),
        issue: Invalid,
    }]);
}

/// Gets the measurements for the given meter.
/// By default (If the input query doesn't specify otherwise), it gets one month
/// of measurements grouped by hour
pub async fn get_measurements_for_meter(
    State(state): State<AppState>,
    Path(meter_id): Path<String>,
    user: Extension<User>,
    Query(input): Query<GetMeasurementsInput>,
) -> Result<Extractor<Series>, AppError> {
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

    let from;
    let to;
    let granularity = input.granularity.unwrap_or(SeriesGranularity::Day);
    match granularity {
        SeriesGranularity::Hour => {
            if input.day.is_none() {
                return Err(AppError::ValidationError(vec![FailedValidation {
                    field: "day".to_string(),
                    issue: Required,
                }]));
            }

            from = input.day.unwrap().and_time(NaiveTime::MIN);
            to = from + Duration::hours(24);
        }
        _ => {
            to = Utc::now().naive_utc();
            from = to - Duration::days(30);
        }
    }

    let measurements = state
        .storage
        .get_measurements(meter_id, from, to, 2500)
        .await?;

    let ret = create_series(&measurements, granularity);

    Ok(Extractor(ret))
}
