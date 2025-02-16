use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use tracing::debug;
use tracing_subscriber::filter::LevelFilter;

/// Middleware that logs the request body on debug level
pub async fn debug_request(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
    if LevelFilter::current() < LevelFilter::DEBUG {
        Ok(next.run(request).await)
    } else {
        let request = buffer_request_body(request).await?;
        Ok(next.run(request).await)
    }
}

/// The trick is to take the request apart, buffer the body, use it, then put
/// the request back together
async fn buffer_request_body(request: Request) -> Result<Request, Response> {
    let (parts, body) = request.into_parts();

    // This won't work if the body is an long running stream
    let bytes = body
        .collect()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?
        .to_bytes();

    debug!(body = ?bytes);

    Ok(Request::from_parts(parts, Body::from(bytes)))
}
