use crate::{
    api::{common::PaginatedRequest, health::Health},
    error::app_error::{bad_request, internal_error, AppError},
    json::extractor::Extractor,
    AppState,
};

use axum::extract::State;
use chrono::{Duration, NaiveDateTime, Utc};
use tracing::error;

pub const ALERTS_MAX_FREQUENCY_MINS: &'static i64 = &20;
pub const DT_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S%.f";
pub const LAST_ALERTS_RUN_KEY: &'static str = "last-alerts-run";
pub const MEASUREMENTS_PAGE_SIZE: &'static u8 = &10;
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

        has_more = meters.pagination.has_more;
        options.page_cursor = Some(meters.items.last().unwrap().id.clone());
        for m in meters.items {
            let since = Utc::now().naive_utc() - Duration::hours(2);
            let measurements = match state
                .storage
                .get_measurements(m.id.clone(), since, *MEASUREMENTS_PAGE_SIZE as u32)
                .await
            {
                Ok(m) => m,
                Err(e) => {
                    error!("Error triggering alerts: {}", e);
                    return internal_error();
                }
            };

            if state.alert_helper.has_constant_flow(&measurements) {
                let account = match state.storage.user_by_id(&m.owner_id).await {
                    Ok(a) => {
                        if a.is_none() {
                            error!("Meter assigned to account, but account not found. Meter: {} Account: {}", &m.id, &m.owner_id);
                            return internal_error();
                        }
                        a.unwrap()
                    }
                    Err(e) => {
                        error!("Error triggering alerts: {}", e);
                        return internal_error();
                    }
                };

                if !state
                    .mail_helper
                    .constant_flow_alert(&state.settings, &account, &m)
                    .await
                {
                    error!("Error sending alert e-mail for meter: {}", &m.id);
                }
            }

            if state.alert_helper.isnt_reporting(&m, &measurements) {
                let account = match state.storage.user_by_id(&m.owner_id).await {
                    Ok(a) => {
                        if a.is_none() {
                            error!("Meter assigned to account, but account not found. Meter: {} Account: {}", &m.id, &m.owner_id);
                            return internal_error();
                        }
                        a.unwrap()
                    }
                    Err(e) => {
                        error!("Error triggering alerts: {}", e);
                        return internal_error();
                    }
                };

                if !state
                    .mail_helper
                    .not_reporting_alert(&state.settings, &account, &m)
                    .await
                {
                    error!("Error sending alert e-mail for meter: {}", &m.id);
                }
            }
        }
    }

    Ok(Extractor(Health {}))
}
