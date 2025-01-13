use crate::{helper::token::AUTH_TOKEN_LEN, AppState};

use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use http::method::Method;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use tracing::error;

static PUBLIC_PATHS: Lazy<HashMap<&str, HashSet<Method>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("/health", HashSet::from([Method::GET]));
    m.insert("/v1/email-verification", HashSet::from([Method::GET]));
    m.insert("/v1/log-in", HashSet::from([Method::POST]));
    m.insert("/v1/measurement", HashSet::from([Method::POST]));
    m.insert("/v1/sign-up", HashSet::from([Method::POST]));
    return m;
});

pub async fn auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method();
    let path = request.uri().path();
    if PUBLIC_PATHS.contains_key(path) && PUBLIC_PATHS.get(path).unwrap().contains(method) {
        return Ok(next.run(request).await);
    }

    let auth_header = match request.headers().get(AUTHORIZATION) {
        Some(h) => h.to_str().unwrap(),
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    let parts: Vec<&str> = auth_header.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match parts[0] {
        "BEARER" => {
            let token = parts[1];
            if token.len() < *AUTH_TOKEN_LEN {
                return Err(StatusCode::UNAUTHORIZED);
            }
            match state.storage.user_by_token(token).await {
                Ok(u) => {
                    if u.is_none() {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                    let mut user = u.unwrap();
                    user.password = None;
                    request.extensions_mut().insert(user);
                    return Ok(next.run(request).await);
                }
                Err(e) => {
                    error!("Failed to authorize user: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        _ => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
}
