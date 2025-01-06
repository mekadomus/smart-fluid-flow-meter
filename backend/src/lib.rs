mod handler;
mod json;

pub mod api;
pub mod error;
pub mod helper;
pub mod http_client;
pub mod settings;
pub mod storage;

use crate::handler::health::health_check;
use crate::handler::measurement::save_measurement;
use crate::handler::user::sign_up_user;
use crate::helper::user::UserHelper;
use crate::settings::settings::Settings;
use crate::storage::Storage;

use axum::{
    extract::FromRef,
    http::{header::HeaderValue, Method},
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
struct AppState {
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
    settings: Arc<Settings>,
    storage: Arc<dyn Storage>,
    user_helper: Arc<dyn UserHelper>,
) -> Router {
    let state = AppState {
        settings: settings.clone(),
        storage,
        user_helper,
    };

    let cors = create_cors_layer(settings);
    Router::new()
        .route("/health", get(health_check))
        .route("/v1/measurement", post(save_measurement))
        .route("/v1/sign-up", post(sign_up_user))
        .with_state(state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}
