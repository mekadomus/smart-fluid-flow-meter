use smart_fluid_flow_meter_backend::{
    api::measurement::SaveMeasurementInput,
    helper::user::MockUserHelper,
    settings::settings::Settings,
    storage::{firestore::FirestoreStorage, memory::MemoryStorage, Storage},
};

use axum::{
    body::Body,
    http,
    http::{Request, StatusCode},
    Router,
};
use chrono::{DateTime, Local};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use test_log::test;
use tower::util::ServiceExt;

async fn create_memory_app() -> Router {
    let settings = Arc::new(Settings::from_file(
        "/smart-fluid-flow-meter/tests/config/default.yaml",
    ));
    let storage = Arc::new(MemoryStorage::new().await);
    let user_helper = Arc::new(MockUserHelper::new());
    return smart_fluid_flow_meter_backend::app(settings, storage, user_helper).await;
}

#[tokio::test]
async fn save_measurement_invalid_json() {
    let app = create_memory_app().await;
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
        json!({ "code": "InvalidInput", "data": "", "message": "Invalid input" })
    );
}

#[tokio::test]
async fn save_measurement_success() {
    let app = create_memory_app().await;
    let input = SaveMeasurementInput {
        device_id: "999".to_string(),
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
    assert_eq!(body.get("id").unwrap().as_str().unwrap(), input.device_id);
    assert_eq!(
        body.get("device_id").unwrap().as_str().unwrap(),
        input.device_id
    );
    assert_eq!(
        body.get("measurement").unwrap().as_str().unwrap(),
        input.measurement
    );
    let actual_date =
        DateTime::parse_from_rfc3339(body.get("recorded_at").unwrap().as_str().unwrap())
            .expect("Bad date");
    assert!(Local::now().timestamp_nanos_opt() > actual_date.timestamp_nanos_opt());
}

#[tokio::test]
async fn save_measurement_database_failure() {
    let app = create_memory_app().await;

    // There will be a failure because device_id is empty
    let input = SaveMeasurementInput {
        device_id: "".to_string(),
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
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "code": "InternalError", "data": "", "message": "We made a mistake. Sorry" })
    );
}

#[test(tokio::test)]
async fn save_measurement_success_firestore() {
    let settings = Arc::new(Settings::from_file(
        "/smart-fluid-flow-meter/tests/config/default.yaml",
    ));
    let storage = Arc::new(FirestoreStorage::new("dummy-id", "db-id").await);
    let user_helper = Arc::new(MockUserHelper::new());
    let app = smart_fluid_flow_meter_backend::app(settings, storage, user_helper).await;

    let input = SaveMeasurementInput {
        device_id: "999".to_string(),
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
    let actual_date =
        DateTime::parse_from_rfc3339(body.get("recorded_at").unwrap().as_str().unwrap());
    assert!(
        Local::now().timestamp_nanos_opt() > actual_date.expect("Bad date").timestamp_nanos_opt()
    );
}

#[tokio::test]
async fn save_measurement_ignores_duplicate_firestore() {
    let settings = Arc::new(Settings::from_file(
        "/smart-fluid-flow-meter/tests/config/default.yaml",
    ));
    let storage = Arc::new(FirestoreStorage::new("dummy-id", "db-id").await);
    let user_helper = Arc::new(MockUserHelper::new());
    let app = smart_fluid_flow_meter_backend::app(settings, storage.clone(), user_helper).await;

    let input = SaveMeasurementInput {
        device_id: "666".to_string(),
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

    assert_eq!(response.status(), StatusCode::OK);

    match storage
        .get_measurements("666".to_string(), Local::now(), 10)
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
