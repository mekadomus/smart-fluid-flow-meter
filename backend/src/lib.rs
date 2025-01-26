mod handler;
mod json;

pub mod api;
pub mod error;
pub mod helper;
pub mod http_client;
pub mod middleware;
pub mod settings;
pub mod storage;

use crate::{
    handler::{
        fluid_meter::{create_fluid_meter, fluid_meters},
        health::health_check,
        measurement::save_measurement,
        user::{email_verification, log_in_user, log_out_user, me, sign_up_user},
    },
    helper::{mail::MailHelper, user::UserHelper},
    middleware::auth::{auth, Authorizer},
    settings::settings::Settings,
    storage::Storage,
};

use axum::{
    extract::FromRef,
    http::{header::HeaderValue, Method},
    middleware::from_fn_with_state,
    routing::get,
    routing::post,
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::{error, info, Level};

#[derive(Clone, FromRef)]
pub struct AppState {
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
    authorizer: Arc<dyn Authorizer>,
    mail_helper: Arc<dyn MailHelper>,
    settings: Arc<Settings>,
    storage: Arc<dyn Storage>,
    user_helper: Arc<dyn UserHelper>,
) -> Router {
    let state = AppState {
        authorizer,
        mail_helper,
        settings: settings.clone(),
        storage,
        user_helper,
    };

    let cors = create_cors_layer(settings);
    Router::new()
        .route("/health", get(health_check))
        .route("/v1/email-verification", get(email_verification))
        .route("/v1/fluid-meter", get(fluid_meters))
        .route("/v1/fluid-meter", post(create_fluid_meter))
        .route("/v1/log-in", post(log_in_user))
        .route("/v1/log-out", post(log_out_user))
        .route("/v1/me", get(me))
        .route("/v1/measurement", post(save_measurement))
        .route("/v1/sign-up", post(sign_up_user))
        .with_state(state.clone())
        .layer(from_fn_with_state(state, auth))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}
