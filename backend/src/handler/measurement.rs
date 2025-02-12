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
    error::app_error::{validation_error, AppError, FailedValidation, ValidationIssue::Invalid},
    json::extractor::Extractor,
    AppState,
};

pub async fn save_measurement(
    State(state): State<AppState>,
    user: Extension<User>,
    Extractor(input): Extractor<SaveMeasurementInput>,
) -> Result<Extractor<Measurement>, AppError> {
    let meter = state
        .storage
        .get_fluid_meter_by_id(&input.device_id)
        .await?;
    match meter {
        Some(m) => {
            if m.owner_id == user.id && m.status == Active {
                let measurement = Measurement {
                    id: None,
                    device_id: input.device_id,
                    measurement: input.measurement,
                    recorded_at: Utc::now().naive_utc(),
                };
                let inserted = state.storage.save_measurement(measurement).await?;

                return Ok(Extractor(Measurement {
                    id: inserted.id,
                    device_id: inserted.device_id,
                    measurement: inserted.measurement,
                    recorded_at: inserted.recorded_at,
                }));
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
                preriod_start: current_start,
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
    let mut validation_errors = vec![];
    if Uuid::try_parse(&meter_id).is_err() {
        validation_errors.push(FailedValidation {
            field: "meter_id".to_string(),
            issue: Invalid,
        });
    } else {
        if !state
            .storage
            .is_fluid_meter_owner(&meter_id, &user.id)
            .await?
        {
            validation_errors.push(FailedValidation {
                field: "meter_id".to_string(),
                issue: Invalid,
            });
        }
    }

    if !validation_errors.is_empty() {
        return Err(AppError::ValidationError(validation_errors));
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
                id: None,
                device_id: "device_id".to_string(),
                measurement: 5.5.to_string(),
                recorded_at: now,
            },
            Measurement {
                id: None,
                device_id: "device_id".to_string(),
                measurement: 1.to_string(),
                recorded_at: now - Duration::minutes(40),
            },
            Measurement {
                id: None,
                device_id: "device_id".to_string(),
                measurement: 22.3.to_string(),
                recorded_at: now - Duration::minutes(60),
            },
            Measurement {
                id: None,
                device_id: "device_id".to_string(),
                measurement: 1.to_string(),
                recorded_at: now - Duration::minutes(80),
            },
            Measurement {
                id: None,
                device_id: "device_id".to_string(),
                measurement: 2.to_string(),
                recorded_at: now - Duration::minutes(100),
            },
            Measurement {
                id: None,
                device_id: "device_id".to_string(),
                measurement: 3.to_string(),
                recorded_at: now - Duration::minutes(120),
            },
        ];
        let res = create_series(&measuments, &one_month_ago);

        let mut items = vec![];
        let mut hour = now - Duration::hours(1);
        items.push(SeriesItem {
            preriod_start: hour,
            value: "6.5".to_string(),
        });
        hour = hour - Duration::hours(1);
        items.push(SeriesItem {
            preriod_start: hour,
            value: "25.3".to_string(),
        });
        hour = hour - Duration::hours(1);
        items.push(SeriesItem {
            preriod_start: hour,
            value: "3".to_string(),
        });
        let expected = Series {
            granularity: SeriesGranularity::Hour,
            items,
        };
        assert_eq!(expected, res);
    }
}
