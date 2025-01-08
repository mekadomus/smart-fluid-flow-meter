use smart_fluid_flow_meter_backend::{
    api::user::SignUpUserInput,
    error::app_error::AppError,
    helper::user::{MockUserHelper, UserHelper},
    settings::settings::Settings,
    storage::firestore::FirestoreStorage,
};

use axum::{
    body::Body,
    http,
    http::{Request, StatusCode},
    Router,
};
use chrono::{DateTime, Local};
use http_body_util::BodyExt;
use mockall::predicate::eq;
use serde_json::{json, Value};
use std::sync::Arc;
use test_log::test;
use tower::util::ServiceExt;

async fn create_app(user_helper: Arc<dyn UserHelper>) -> Router {
    let settings = Arc::new(Settings::from_file(
        "/smart-fluid-flow-meter/tests/config/default.yaml",
    ));
    let storage = Arc::new(FirestoreStorage::new("dummy-id", "db-id").await);
    return smart_fluid_flow_meter_backend::app(settings, storage.clone(), user_helper).await;
}

#[test(tokio::test)]
async fn sign_up_user_weak_password() {
    let password = "12345678";
    let captcha = "heyyou";
    let hashed_password = "hashed-123";

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_password_is_weak()
        .with(eq(password))
        .return_const(true);
    user_helper_mock
        .expect_is_bot()
        .with(eq("my_secret"), eq(captcha), eq("userip"))
        .return_const(false);
    user_helper_mock
        .expect_hash()
        .with(eq(password))
        .returning(|_| Ok(hashed_password.to_string()));

    let app = create_app(Arc::new(user_helper_mock)).await;

    let input = SignUpUserInput {
        email: "my.user@you.know".to_string(),
        name: "Someone last".to_string(),
        password: password.to_string(),
        captcha: captcha.to_string(),
    };
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/sign-up")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "code": "ValidationError", "data": { "ValidationInfo": [ { "field": "password", "issue": "TooWeak" } ] }, "message": "Request data is invalid" })
    );
}

#[test(tokio::test)]
async fn sign_up_failed_captcha() {
    let password = "12345678";
    let captcha = "heyyou";

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_password_is_weak()
        .with(eq(password))
        .return_const(false);
    user_helper_mock
        .expect_is_bot()
        .with(eq("my_secret"), eq(captcha), eq("userip"))
        .return_const(true);

    let app = create_app(Arc::new(user_helper_mock)).await;

    let input = SignUpUserInput {
        email: "my.user@you.know".to_string(),
        name: "Someone last".to_string(),
        password: password.to_string(),
        captcha: captcha.to_string(),
    };
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/sign-up")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "code": "ValidationError", "data": { "ValidationInfo": [ { "field": "captcha", "issue": "Invalid" } ] }, "message": "Request data is invalid" })
    );
}

#[test(tokio::test)]
async fn sign_up_failed_hash() {
    let password = "12345678";
    let captcha = "heyyou";

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_password_is_weak()
        .with(eq(password))
        .return_const(false);
    user_helper_mock
        .expect_is_bot()
        .with(eq("my_secret"), eq(captcha), eq("userip"))
        .return_const(false);
    user_helper_mock
        .expect_hash()
        .with(eq(password))
        .returning(|_| Err(AppError::ServerError));

    let app = create_app(Arc::new(user_helper_mock)).await;

    let input = SignUpUserInput {
        email: "my.user@you.know".to_string(),
        name: "Someone last".to_string(),
        password: password.to_string(),
        captcha: captcha.to_string(),
    };
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/sign-up")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "code": "InternalError", "data": "", "message": "We made a mistake. Sorry" })
    );
}

#[test(tokio::test)]
async fn sign_up_failed_empty_name() {
    let password = "12345678";
    let captcha = "heyyou";
    let hashed_password = "hashed-123";

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_password_is_weak()
        .with(eq(password))
        .return_const(false);
    user_helper_mock
        .expect_is_bot()
        .with(eq("my_secret"), eq(captcha), eq("userip"))
        .return_const(false);
    user_helper_mock
        .expect_hash()
        .with(eq(password))
        .returning(|_| Ok(hashed_password.to_string()));

    let app = create_app(Arc::new(user_helper_mock)).await;

    let input = SignUpUserInput {
        email: "my.user@you.know".to_string(),
        name: "    ".to_string(),
        password: password.to_string(),
        captcha: captcha.to_string(),
    };
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/sign-up")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "code": "ValidationError", "data": { "ValidationInfo": [ { "field": "name", "issue": "Required" } ] }, "message": "Request data is invalid" })
    );
}

#[test(tokio::test)]
async fn sign_up_failed_invalid_email() {
    let password = "12345678";
    let captcha = "heyyou";
    let hashed_password = "hashed-123";

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_password_is_weak()
        .with(eq(password))
        .return_const(false);
    user_helper_mock
        .expect_is_bot()
        .with(eq("my_secret"), eq(captcha), eq("userip"))
        .return_const(false);
    user_helper_mock
        .expect_hash()
        .with(eq(password))
        .returning(|_| Ok(hashed_password.to_string()));

    let app = create_app(Arc::new(user_helper_mock)).await;

    let input = SignUpUserInput {
        email: "my.useryou.know".to_string(),
        name: "Gallo Claudio".to_string(),
        password: password.to_string(),
        captcha: captcha.to_string(),
    };
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/sign-up")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "code": "ValidationError", "data": { "ValidationInfo": [ { "field": "email", "issue": "Invalid" } ] }, "message": "Request data is invalid" })
    );
}

#[test(tokio::test)]
async fn sign_up_user_success() {
    let password = "12345678";
    let captcha = "heyyou";
    let hashed_password = "hashed-123";

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_password_is_weak()
        .with(eq(password))
        .return_const(false);
    user_helper_mock
        .expect_is_bot()
        .with(eq("my_secret"), eq(captcha), eq("userip"))
        .return_const(false);
    user_helper_mock
        .expect_hash()
        .with(eq(password))
        .returning(|_| Ok(hashed_password.to_string()));

    let app = create_app(Arc::new(user_helper_mock)).await;

    let input = SignUpUserInput {
        email: "my.user@you.know".to_string(),
        name: "Someone last".to_string(),
        password: password.to_string(),
        captcha: captcha.to_string(),
    };
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/sign-up")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body.get("id").unwrap().as_str().unwrap(),
        "my.user@you.know+password"
    );
    assert_eq!(body.get("provider").unwrap().as_str().unwrap(), "password");
    assert_eq!(body.get("email").unwrap().as_str().unwrap(), input.email);
    assert!(body.get("password").is_none()); // Password shouldn't be returned
    let actual_date =
        DateTime::parse_from_rfc3339(body.get("recorded_at").unwrap().as_str().unwrap());
    assert!(
        Local::now().timestamp_nanos_opt() > actual_date.expect("Bad date").timestamp_nanos_opt()
    );
    // E-mail hasn't been verified
    assert!(body.get("email_verified_at").unwrap().as_str().is_none());
}

#[test(tokio::test)]
async fn sign_up_user_duplicate() {
    let password = "0987654321";
    let captcha = "hellothere";
    let hashed_password = "hashed-123";

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_password_is_weak()
        .with(eq(password))
        .return_const(false);
    user_helper_mock
        .expect_is_bot()
        .with(eq("my_secret"), eq(captcha), eq("userip"))
        .return_const(false);
    user_helper_mock
        .expect_hash()
        .with(eq(password))
        .returning(|_| Ok(hashed_password.to_string()));

    let app = create_app(Arc::new(user_helper_mock)).await;

    let input = SignUpUserInput {
        email: "other.user@you.know".to_string(),
        name: "Someone last".to_string(),
        password: password.to_string(),
        captcha: captcha.to_string(),
    };
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/sign-up")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/sign-up")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "code": "InternalError", "data": "", "message": "We made a mistake. Sorry" })
    );
}
