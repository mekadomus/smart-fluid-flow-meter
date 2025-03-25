mod handler;
mod json;

pub mod api;
pub mod error;
pub mod helper;
pub mod http_client;
pub mod logging;
pub mod middleware;
pub mod settings;
pub mod storage;

use axum::{
    extract::FromRef,
    http::{header::HeaderValue, Method},
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::{error, info, Level};

use crate::{
    handler::{
        alert::trigger_alerts,
        fluid_meter::{
            activate_fluid_meter,
            create_fluid_meter,
            deactivate_fluid_meter,
            fluid_meters,
            get_fluid_meter, //, get_fluid_meter_alerts
        },
        health::health_check,
        measurement::{get_measurements_for_meter, save_measurement},
        user::{
            email_verification, log_in_user, log_out_user, me, new_password, recover_password,
            sign_up_user,
        },
    },
    helper::{alert::AlertHelper, mail::MailHelper, user::UserHelper},
    middleware::{
        auth::{auth, Authorizer},
        debug_request::debug_request,
    },
    settings::settings::{LoggingFormat::Json, Settings},
    storage::Storage,
};

#[derive(Clone, FromRef)]
pub struct AppState {
    alert_helper: Arc<dyn AlertHelper>,
    authorizer: Arc<dyn Authorizer>,
    mail_helper: Arc<dyn MailHelper>,
    settings: Arc<Settings>,
    storage: Arc<dyn Storage>,
    user_helper: Arc<dyn UserHelper>,
}

fn create_cors_layer(settings: Arc<Settings>) -> CorsLayer {
    let mut cors_domains = vec![HeaderValue::from_static("http://localhost:5173")];

    let parts = settings.service.cors_domains.split(",");
    for domain in parts {
        match HeaderValue::from_str(domain) {
            Ok(h) => {
                cors_domains.push(h);
                info!("Added domain {} to cors", domain);
            }
            Err(e) => {
                error!("Couldn't add domain {} to cors. {}", domain, e);
            }
        };
    }

    return CorsLayer::new()
        .allow_methods([Method::DELETE, Method::GET, Method::POST, Method::PUT])
        .allow_headers([
            "Content-Type".parse().unwrap(),
            "Authorization".parse().unwrap(),
        ])
        .allow_origin(AllowOrigin::list(cors_domains));
}

pub async fn app(
    alert_helper: Arc<dyn AlertHelper>,
    authorizer: Arc<dyn Authorizer>,
    mail_helper: Arc<dyn MailHelper>,
    settings: Arc<Settings>,
    storage: Arc<dyn Storage>,
    user_helper: Arc<dyn UserHelper>,
) -> Router {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(settings.logging.level.0)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true);
    if settings.logging.format == Json {
        let _ = subscriber.json().try_init();
    } else {
        let _ = subscriber.try_init();
    }

    let state = AppState {
        alert_helper,
        authorizer,
        mail_helper,
        settings: settings.clone(),
        storage,
        user_helper,
    };

    let cors = create_cors_layer(settings);
    Router::new()
        .route("/health", get(health_check))
        // Users and sessions
        .route("/v1/email-verification", get(email_verification))
        .route("/v1/log-in", post(log_in_user))
        .route("/v1/log-out", post(log_out_user))
        .route("/v1/me", get(me))
        .route("/v1/new-password", post(new_password))
        .route("/v1/recover-password", get(recover_password))
        .route("/v1/sign-up", post(sign_up_user))
        // Fluid meters
        .route("/v1/fluid-meter", get(fluid_meters))
        .route("/v1/fluid-meter", post(create_fluid_meter))
        .route("/v1/fluid-meter/{meter_id}", get(get_fluid_meter))
        .route(
            "/v1/fluid-meter/{meter_id}/activate",
            post(activate_fluid_meter),
        )
        // .route("/v1/fluid-meter/{meter_id}/alert", get(get_fluid_meter_alerts))
        .route(
            "/v1/fluid-meter/{meter_id}/deactivate",
            post(deactivate_fluid_meter),
        )
        .route(
            "/v1/fluid-meter/{meter_id}/measurement",
            get(get_measurements_for_meter),
        )
        // Measurements
        .route("/v1/measurement", post(save_measurement))
        // Alerts
        .route("/v1/alert", post(trigger_alerts))
        .with_state(state.clone())
        .layer(from_fn_with_state(state, auth))
        .layer(from_fn(debug_request))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}
