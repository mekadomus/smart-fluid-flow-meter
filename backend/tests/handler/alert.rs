use axum::{
    body::Body,
    http,
    http::{Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use http::header::CONTENT_TYPE;
use mockall::predicate::{always, eq};
use std::sync::Arc;
use test_log::test;
use tower::util::ServiceExt;
use uuid::Uuid;

use smart_fluid_flow_meter_backend::{
    api::measurement::Measurement,
    helper::mail::MockMailHelper,
    storage::{postgres::PostgresStorage, MeasurementStorage},
};

use crate::helper::{app::create_app_mail_helper, fluid_meter::create_fluid_meter};

#[test(tokio::test)]
async fn alert_success() {
    let storage = PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await;

    // Fluid meter with alert
    let fm = create_fluid_meter().await;
    let mut m = Measurement {
        id: Uuid::new_v4().to_string(),
        device_id: fm.id.clone(),
        measurement: "1".to_string(),
        recorded_at: Utc::now().naive_utc() - Duration::minutes(80),
    };
    let _ = storage.save_measurement(&m).await;
    m.id = Uuid::new_v4().to_string();
    m.recorded_at = m.recorded_at + Duration::minutes(20);
    let _ = storage.save_measurement(&m).await;
    m.id = Uuid::new_v4().to_string();
    m.recorded_at = m.recorded_at + Duration::minutes(20);
    let _ = storage.save_measurement(&m).await;
    m.id = Uuid::new_v4().to_string();
    m.recorded_at = m.recorded_at + Duration::minutes(20);
    let _ = storage.save_measurement(&m).await;
    m.id = Uuid::new_v4().to_string();
    m.recorded_at = m.recorded_at + Duration::minutes(20);
    let _ = storage.save_measurement(&m).await;

    // Fluid meter without alert
    let fm2 = create_fluid_meter().await;
    let mut m = Measurement {
        id: Uuid::new_v4().to_string(),
        device_id: fm2.id,
        measurement: "1".to_string(),
        recorded_at: Utc::now().naive_utc() + Duration::minutes(80),
    };
    let _ = storage.save_measurement(&m).await;
    m.id = Uuid::new_v4().to_string();
    m.recorded_at = m.recorded_at + Duration::minutes(20);
    m.measurement = "0".to_string();
    let _ = storage.save_measurement(&m).await;
    m.id = Uuid::new_v4().to_string();
    m.recorded_at = m.recorded_at + Duration::minutes(20);
    m.measurement = "1".to_string();
    let _ = storage.save_measurement(&m).await;
    m.id = Uuid::new_v4().to_string();
    m.recorded_at = m.recorded_at + Duration::minutes(20);
    let _ = storage.save_measurement(&m).await;
    m.id = Uuid::new_v4().to_string();
    m.recorded_at = m.recorded_at + Duration::minutes(20);
    let _ = storage.save_measurement(&m).await;

    // Expect alert email submission
    let mut mail_helper_mock = MockMailHelper::new();
    mail_helper_mock
        .expect_constant_flow_alert()
        .with(always(), always(), eq(fm))
        .return_const(true);

    let app = create_app_mail_helper(Arc::new(mail_helper_mock)).await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/alert")
                .header(CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
