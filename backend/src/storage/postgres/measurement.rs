use async_trait::async_trait;
use chrono::{Duration, NaiveDateTime, Utc};
use tracing::error;

use crate::{
    api::measurement::Measurement,
    storage::{
        error::{rate_limit, undefined, Error, ErrorCode},
        postgres::PostgresStorage,
        MeasurementStorage,
    },
};

#[async_trait]
impl MeasurementStorage for PostgresStorage {
    // Measurements are rate-limitted by device to prevent devices from spamming us
    async fn save_measurement(&self, measurement: &Measurement) -> Result<Measurement, Error> {
        let mut tx = match self.pool.begin().await {
            Ok(t) => t,
            Err(e) => {
                error!("Error creating transaction. {}", e);
                return undefined();
            }
        };

        let last: Option<Measurement> = match sqlx::query_as(
            r#"
            SELECT *
            FROM measurement
            WHERE device_id = $1
            ORDER BY recorded_at DESC
            LIMIT 1
            "#,
        )
        .bind(&measurement.device_id)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(m) => Some(m),
            Err(e) => match e {
                sqlx::Error::RowNotFound => None,
                _ => {
                    error!("Error getting measurements for device. {}", e);
                    return undefined();
                }
            },
        };

        if last.is_some()
            && last.unwrap().recorded_at > Utc::now().naive_utc() - Duration::minutes(10)
        {
            error!("Rate limiting device: {}", measurement.device_id);
            return rate_limit();
        }

        match sqlx::query("INSERT INTO measurement(id, device_id, measurement, recorded_at) VALUES($1, $2, $3, $4)")
        .bind(&measurement.id)
        .bind(&measurement.device_id)
        .bind(&measurement.measurement)
        .bind(&measurement.recorded_at)
        .execute(&mut *tx)
        .await
        {
            Ok(_) => {},
            Err(e) => {
                error!("Error: {}", e);
                return undefined()
            }
        }

        let _ = tx.commit().await;
        return Ok(measurement.clone());
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
