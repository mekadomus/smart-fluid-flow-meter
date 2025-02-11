use async_trait::async_trait;
use chrono::{NaiveDateTime, Timelike};
use tracing::{error, info};

use crate::{
    api::measurement::Measurement,
    storage::{
        error::{Error, ErrorCode},
        postgres::{PostgresStorage, UNIQUE_VIOLATION},
        MeasurementStorage,
    },
};

#[async_trait]
impl MeasurementStorage for PostgresStorage {
    // The id of the passed measurement is ignored. An id will be assigned
    // automatically
    //
    // We use <device-id>-<datetime-to-minute-precision> as id and we only allow
    // saving at even numbered minutes, so we round the minute down.
    //
    // Examples:
    // device_id: 1234 recorded_at: 2024-12-28T22:08:37.519762489Z document_id: 1234-2024-12-28T22:08
    // device_id: 1234 recorded_at: 2024-12-28T22:09:37.519762489Z document_id: 1234-2024-12-28T22:08
    // (Note how the id uses the minute 08 even when recorded_at is at 09)
    //
    // We do this because the firmware sometimes sends duplicated requests a few
    // miliseconds apart. By always rounding down to an even minute we make
    // sure that duplicated requests will always have the same id, so won't be
    // accepted
    async fn save_measurement(&self, measurement: Measurement) -> Result<Measurement, Error> {
        let mut recorded_at = measurement.recorded_at;
        let minute = recorded_at.minute();
        if minute % 2 != 0 {
            recorded_at = recorded_at.with_minute(minute - 1).unwrap();
        }
        let time = recorded_at.format("%Y-%m-%dT%H:%M").to_string();
        let id = format!("{}-{}", measurement.device_id.clone(), time);

        match sqlx::query("INSERT INTO measurement(id, device_id, measurement, recorded_at) VALUES($1, $2, $3, $4)")
        .bind(&id)
        .bind(&measurement.device_id)
        .bind(&measurement.measurement)
        .bind(recorded_at)
        .execute(&self.pool)
        .await
        {
            Ok(_) => return Ok(Measurement{
                id: Some(id),
                recorded_at,
                ..measurement
            }),
            Err(e) => {
                match e {
                    sqlx::Error::Database(ref e) => {
                        if let Some(UNIQUE_VIOLATION) = e.code().as_deref() {
                            info!("Trying to insert already existing measurement: {}", &id);
                            return Ok(Measurement{
                                id: Some(id),
                                recorded_at,
                                ..measurement
                            })
                        }
                    },
                    _ => {}
                }
                error!("Error: {}", e);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        };
    }

    async fn get_measurements(
        &self,
        device_id: String,
        since: NaiveDateTime,
        num_records: u32,
    ) -> Result<Vec<Measurement>, Error> {
        match sqlx::query_as(
            r#"
            SELECT
                id,
                device_id,
                measurement,
                recorded_at
            FROM measurement
            WHERE
                device_id = $1 AND
                recorded_at <= $2
            LIMIT $3
        "#,
        )
        .bind(device_id.clone())
        .bind(since)
        .bind(num_records as i32)
        .fetch_all(&self.pool)
        .await
        {
            Ok(found) => {
                return Ok(found);
            }
            Err(err) => {
                error!("Error: {}", err);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        };
    }
}
