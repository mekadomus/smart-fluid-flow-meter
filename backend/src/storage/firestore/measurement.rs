use async_trait::async_trait;
use chrono::{DateTime, Local, Timelike};
use firestore::{errors::FirestoreError::DataConflictError, path, FirestoreQueryDirection};
use tracing::{error, info};

use crate::{
    api::measurement::Measurement,
    storage::{
        error::{Error, ErrorCode},
        firestore::FirestoreStorage,
        MeasurementStorage,
    },
};

const MEASUREMENT_COLLECTION: &'static str = "measurement";

#[async_trait]
impl MeasurementStorage for FirestoreStorage {
    // The id of the passed measurement is ignored. An id will be assigned automtically
    // We only allow saving at most one
    // Because of limitations of firestore, this method does some funky stuff
    // We use <device-id>-<datetime-to-minute-precision> as document id and we
    // only allow saving at even numbered minutes, so we round the minute down.
    //
    // Examples:
    // device_id: 1234 recorded_at: 2024-12-28T22:08:37.519762489Z document_id: 1234-2024-12-28T22:08
    // device_id: 1234 recorded_at: 2024-12-28T22:09:37.519762489Z document_id: 1234-2024-12-28T22:08
    // (Note how the document_id uses the minute 08 even when recorded_at is at 09)
    //
    // We do this because the firmware sometimes sends duplicated requests a few
    // miliseconds apart by using always rounding down to an even minute we make
    // sure that duplicated requests will always have the same id, so won't be
    // accepted
    async fn save_measurement(&self, measurement: Measurement) -> Result<Measurement, Error> {
        let mut recorded_at = measurement.recorded_at;
        let minute = recorded_at.minute();
        if minute % 2 != 0 {
            recorded_at = recorded_at.with_minute(minute - 1).unwrap();
        }
        let time = recorded_at.format("%Y-%m-%dT%H:%M").to_string();
        let document_id = format!("{}-{}", measurement.device_id, time);
        let inserted: Measurement = match self
            .db
            .fluent()
            .insert()
            .into(MEASUREMENT_COLLECTION)
            .document_id(&document_id)
            .object(&measurement)
            .execute()
            .await
        {
            Ok(inserted) => inserted,
            // If the record already exists, we just return it
            Err(DataConflictError(_)) => {
                info!(
                    "Trying to insert already existing measurement: {}",
                    &document_id
                );

                match self
                    .db
                    .fluent()
                    .select()
                    .by_id_in(MEASUREMENT_COLLECTION)
                    .obj()
                    .one(&document_id)
                    .await
                {
                    Ok(f) => match f {
                        Some(f) => {
                            return Ok(f);
                        }
                        None => {
                            error!(
                                "Couldn't find already existing measurement: {}",
                                &document_id
                            );
                            return Err(Error {
                                code: ErrorCode::UndefinedError,
                            });
                        }
                    },
                    Err(e) => {
                        error!("Error: {}", e);
                        return Err(Error {
                            code: ErrorCode::UndefinedError,
                        });
                    }
                };
            }
            Err(err) => {
                error!("Error: {}", err);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        };

        Ok(Measurement {
            id: inserted.id,
            ..measurement
        })
    }

    async fn get_measurements(
        &self,
        device_id: String,
        since: DateTime<Local>,
        num_records: u32,
    ) -> Result<Vec<Measurement>, Error> {
        match self
            .db
            .fluent()
            .select()
            .from(MEASUREMENT_COLLECTION)
            .filter(|q| {
                q.for_all([
                    q.field(path!(Measurement::device_id)).eq(device_id.clone()),
                    q.field(path!(Measurement::recorded_at))
                        .less_than_or_equal(since),
                ])
            })
            .order_by([(
                path!(Measurement::recorded_at),
                FirestoreQueryDirection::Descending,
            )])
            .limit(num_records)
            .obj()
            .query()
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
