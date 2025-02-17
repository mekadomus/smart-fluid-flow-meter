use smart_fluid_flow_meter_backend::{
    api::{
        common::{Series, SeriesGranularity::Hour},
        fluid_meter::{
            FluidMeter,
            FluidMeterStatus::{Active, Inactive},
        },
        measurement::SaveMeasurementInput,
        user::{User, UserAuthProvider::Password},
    },
    helper::{mail::MockMailHelper, user::MockUserHelper},
    middleware::auth::MockAuthorizer,
    settings::settings::Settings,
    storage::{postgres::PostgresStorage, FluidMeterStorage, Storage, UserStorage},
};

use axum::{
    body::Body,
    http,
    http::{Request, StatusCode},
    Router,
};
use chrono::{NaiveDateTime, Utc};
use http_body_util::BodyExt;
use mockall::predicate::always;
use serde_json::{json, Value};
use std::sync::Arc;
use test_log::test;
use tower::util::ServiceExt;

pub const DEVICE_ID: &'static str = "3fe50206-25d0-4830-9de1-b48cc2a89001";
pub const DEVICE_ID2: &'static str = "3fe50206-25d0-4830-9de1-b48cc2a89002";
pub const INACTIVE_DEVICE_ID: &'static str = "3fe50206-25d0-4830-9de1-b48cc2a89003";

async fn create_app(with_session: bool) -> (Router, Arc<dyn Storage>) {
    let settings = Arc::new(Settings::from_file(
        "/smart-fluid-flow-meter/tests/config/default.yaml",
    ));
    let storage =
        Arc::new(PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await);

    let user_id = "a@b.c+password";
    let user = User {
        id: user_id.to_string(),
        provider: Password,
        name: "Carlos".to_string(),
        email: "a@b.c".to_string(),
        password: Some("hello".to_string()),
        email_verified_at: Some(Utc::now().naive_utc()),
        recorded_at: Utc::now().naive_utc(),
    };
    let fm = FluidMeter {
        id: DEVICE_ID.to_string(),
        owner_id: user_id.to_string(),
        name: "kitchen".to_string(),
        status: Active,
        recorded_at: Utc::now().naive_utc(),
    };
    let fm2 = FluidMeter {
        id: DEVICE_ID2.to_string(),
        owner_id: user_id.to_string(),
        name: "kitchen".to_string(),
        status: Active,
        recorded_at: Utc::now().naive_utc(),
    };
    let fm3 = FluidMeter {
        id: INACTIVE_DEVICE_ID.to_string(),
        owner_id: user_id.to_string(),
        name: "kitchen".to_string(),
        status: Inactive,
        recorded_at: Utc::now().naive_utc(),
    };
    let _ = storage.insert_user(&user).await;
    let _ = storage.insert_fluid_meter(&fm).await;
    let _ = storage.insert_fluid_meter(&fm2).await;
    let _ = storage.insert_fluid_meter(&fm3).await;

    let mut authorizer = MockAuthorizer::new();
    authorizer
        .expect_authorize()
        .with(always(), always())
        .returning(move |_, request| {
            if with_session {
                request.extensions_mut().insert(user.clone());
            }
            Ok(())
        });

    let user_helper = Arc::new(MockUserHelper::new());
    let mail_helper = Arc::new(MockMailHelper::new());
    return (
        smart_fluid_flow_meter_backend::app(
            Arc::new(authorizer),
            mail_helper,
            settings,
            storage.clone(),
            user_helper,
        )
        .await,
        storage,
    );
}

#[tokio::test]
async fn save_measurement_invalid_json() {
    let (app, _) = create_app(false).await;
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/measurement")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "code": "InvalidInput", "data": "", "message": "Invalid JSON for this endpoint" })
    );
}

#[test(tokio::test)]
async fn save_measurement_not_owner() {
    let (app, _) = create_app(false).await;

    let input = SaveMeasurementInput {
        device_id: "some-dev-id".to_string(),
        measurement: "134".to_string(),
    };
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/measurement")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test(tokio::test)]
async fn save_measurement_inactive() {
    let (app, _) = create_app(false).await;

    let input = SaveMeasurementInput {
        device_id: INACTIVE_DEVICE_ID.to_string(),
        measurement: "134".to_string(),
    };
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/measurement")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test(tokio::test)]
async fn save_measurement_success() {
    let (app, _) = create_app(false).await;

    let input = SaveMeasurementInput {
        device_id: DEVICE_ID.to_string(),
        measurement: "134".to_string(),
    };
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/measurement")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_ne!(body.get("id").unwrap().as_str().unwrap(), "");
    assert_eq!(
        body.get("device_id").unwrap().as_str().unwrap(),
        input.device_id
    );
    assert_eq!(
        body.get("measurement").unwrap().as_str().unwrap(),
        input.measurement
    );
    let actual_date = NaiveDateTime::parse_from_str(
        body.get("recorded_at").unwrap().as_str().unwrap(),
        "%Y-%m-%dT%H:%M:%S%.f",
    );
    assert!(Utc::now().naive_utc() > actual_date.expect("Bad date"));
}

#[tokio::test]
async fn save_measurement_ignores_duplicate() {
    let (app, storage) = create_app(false).await;

    let input = SaveMeasurementInput {
        device_id: DEVICE_ID2.to_string(),
        measurement: "3.781159".to_string(),
    };
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/measurement")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Send a duplicate request
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/measurement")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    match storage
        .get_measurements(DEVICE_ID2.to_string(), Utc::now().naive_utc(), 10)
        .await
    {
        Ok(f) => {
            assert_eq!(f.len(), 1);
        }
        Err(_) => {
            panic!("Error getting measurements from db");
        }
    };
}

#[tokio::test]
async fn get_measurements_for_meter_not_owned() {
    let (app, _) = create_app(true).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/v1/fluid-meter/00000000-0000-0000-0000-000000000000/measurement")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_measurements_for_meter_invalid_meter() {
    let (app, _) = create_app(true).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/v1/fluid-meter/bad-id/measurement")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_measurements_for_meter_success() {
    let (app, _) = create_app(true).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/v1/fluid-meter/{}/measurement", DEVICE_ID))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: Series = serde_json::from_slice(&body).unwrap();

    assert_eq!(resp.granularity, Hour);
    assert_eq!(resp.items.len(), 0);
}
