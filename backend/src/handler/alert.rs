use crate::{
    api::{common::PaginatedRequest, fluid_meter::FluidMeterAlerts, health::Health},
    error::app_error::{bad_request, internal_error, AppError},
    json::extractor::Extractor,
    AppState,
};

use axum::extract::State;
use chrono::{Duration, NaiveDateTime, Utc};
use std::collections::HashMap;
use tracing::error;

pub const ALERTS_MAX_FREQUENCY_MINS: &'static i64 = &20;
pub const DT_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S%.f";
pub const LAST_ALERTS_RUN_KEY: &'static str = "last-alerts-run";
pub const METERS_PAGE_SIZE: &'static u8 = &100;

pub async fn trigger_alerts(State(state): State<AppState>) -> Result<Extractor<Health>, AppError> {
    let last_run = match state.storage.get_metadata(&LAST_ALERTS_RUN_KEY).await {
        Ok(d) => d,
        Err(e) => {
            error!("Error triggering alerts: {}", e);
            return internal_error();
        }
    };

    if last_run.is_some()
        && NaiveDateTime::parse_from_str(&last_run.clone().unwrap().value, DT_FORMAT).unwrap()
            > (Utc::now().naive_utc() - Duration::minutes(*ALERTS_MAX_FREQUENCY_MINS))
    {
        error!(
            "Rate limiting trigger alerts. Last run: {}",
            last_run.unwrap().value
        );
        return bad_request();
    }

    match state
        .storage
        .save_metadata(LAST_ALERTS_RUN_KEY, &Utc::now().naive_utc().to_string())
        .await
    {
        Ok(_) => {}
        Err(e) => {
            error!("Error triggering alerts: {}", e);
            return internal_error();
        }
    };

    let mut has_more = true;
    while has_more {
        let mut options = PaginatedRequest {
            page_cursor: None,
            page_size: *METERS_PAGE_SIZE,
        };
        let meters = match state.storage.get_active_fluid_meters(&options).await {
            Ok(r) => r,
            Err(e) => {
                error!("Error triggering alerts: {}", e);
                return internal_error();
            }
        };

        if meters.items.len() == 0 {
            break;
        }

        // Key is the id of the owner
        let mut alerts: HashMap<String, Vec<FluidMeterAlerts>> = HashMap::new();

        has_more = meters.pagination.has_more;
        options.page_cursor = Some(meters.items.last().unwrap().id.clone());
        for m in meters.items {
            match state
                .alert_helper
                .get_alerts(state.storage.clone(), &m)
                .await
            {
                Ok(a) => {
                    if alerts.contains_key(&m.owner_id) {
                        alerts.get_mut(&m.owner_id).unwrap().push(a);
                    } else {
                        alerts.insert(m.owner_id, vec![a]);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        for a in alerts {
            let account = match state.storage.user_by_id(&a.0).await {
                Ok(ac) => {
                    if ac.is_none() {
                        error!(
                            "Meter assigned to account, but account not found. Account: {}",
                            &a.0
                        );
                        return internal_error();
                    }
                    ac.unwrap()
                }
                Err(e) => {
                    error!("Error triggering alerts: {}", e);
                    return internal_error();
                }
            };

            if !state
                .mail_helper
                .alerts(&state.settings, &account, &a.1)
                .await
            {
                error!("Error sending alerts e-mail for user: {}", &a.0);
            }
        }
    }

    Ok(Extractor(Health {}))
}
