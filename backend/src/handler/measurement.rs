use axum::{
    extract::{Path, State},
    Extension,
};
use chrono::{Duration, NaiveDateTime, Utc};
use uuid::Uuid;

use crate::{
    api::{
        common::{Series, SeriesGranularity, SeriesItem},
        fluid_meter::FluidMeterStatus::Active,
        measurement::{Measurement, SaveMeasurementInput},
        user::User,
    },
    error::app_error::{
        internal_error, validation_error, AppError, FailedValidation,
        ValidationIssue::{Invalid, TooFrequent},
    },
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

fn create_series(measurements: &Vec<Measurement>, start_time: &NaiveDateTime) -> Series {
    let one_month_ago = start_time;
    let mut items = vec![];
    let mut current_start = *start_time + Duration::days(30) - Duration::hours(1);
    let mut i = 0;

    while current_start >= *one_month_ago {
        let mut total = 0.0;
        while i < measurements.len() && measurements[i].recorded_at > current_start {
            total = total + measurements[i].measurement.parse::<f64>().unwrap();
            i = i + 1;
        }

        if total != 0.0 {
            items.push(SeriesItem {
                period_start: current_start,
                value: total.to_string(),
            });
        }
        current_start = current_start - Duration::hours(1);
    }

    return Series {
        granularity: SeriesGranularity::Hour,
        items,
    };
}

/// Gets the measurements for the given meter. It gets one month of measurements
/// grouped by hour
pub async fn get_measurements_for_meter(
    State(state): State<AppState>,
    Path(meter_id): Path<String>,
    user: Extension<User>,
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

    let one_month_ago = Utc::now().naive_utc() - Duration::days(30);
    let measurements = state
        .storage
        .get_measurements(meter_id, one_month_ago, 2500)
        .await?;

    let ret = create_series(&measurements, &one_month_ago);

    Ok(Extractor(ret))
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use super::create_series;
    use crate::api::{
        common::{Series, SeriesGranularity, SeriesItem},
        measurement::Measurement,
    };

    #[test]
    fn create_series_success() {
        let now = Utc::now().naive_utc();
        let one_month_ago = now - Duration::days(30);
        let measuments = vec![
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 5.5.to_string(),
                recorded_at: now,
            },
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 1.to_string(),
                recorded_at: now - Duration::minutes(40),
            },
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 22.3.to_string(),
                recorded_at: now - Duration::minutes(60),
            },
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 1.to_string(),
                recorded_at: now - Duration::minutes(80),
            },
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 2.to_string(),
                recorded_at: now - Duration::minutes(100),
            },
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 3.to_string(),
                recorded_at: now - Duration::minutes(120),
            },
        ];
        let res = create_series(&measuments, &one_month_ago);

        let mut items = vec![];
        let mut hour = now - Duration::hours(1);
        items.push(SeriesItem {
            period_start: hour,
            value: "6.5".to_string(),
        });
        hour = hour - Duration::hours(1);
        items.push(SeriesItem {
            period_start: hour,
            value: "25.3".to_string(),
        });
        hour = hour - Duration::hours(1);
        items.push(SeriesItem {
            period_start: hour,
            value: "3".to_string(),
        });
        let expected = Series {
            granularity: SeriesGranularity::Hour,
            items,
        };
        assert_eq!(expected, res);
    }
}
