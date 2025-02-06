use crate::api::measurement::{Measurement, SaveMeasurementInput};
use crate::error::app_error::AppError;
use crate::json::extractor::Extractor;
use crate::AppState;

use axum::extract::State;
use chrono::Utc;

pub async fn save_measurement(
    State(state): State<AppState>,
    Extractor(input): Extractor<SaveMeasurementInput>,
) -> Result<Extractor<Measurement>, AppError> {
    let measurement = Measurement {
        id: None,
        device_id: input.device_id,
        measurement: input.measurement,
        recorded_at: Utc::now().naive_utc(),
    };
    let inserted = state.storage.save_measurement(measurement).await?;

    Ok(Extractor(Measurement {
        id: inserted.id,
        device_id: inserted.device_id,
        measurement: inserted.measurement,
        recorded_at: inserted.recorded_at,
    }))
}
