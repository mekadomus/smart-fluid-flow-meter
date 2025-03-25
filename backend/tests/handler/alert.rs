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
    api::{
        alert::{Alert, AlertType},
        fluid_meter::FluidMeterAlerts,
        measurement::Measurement,
    },
    helper::mail::MockMailHelper,
    storage::{postgres::PostgresStorage, MeasurementStorage},
};

use crate::helper::{
    app::create_app_mail_helper,
    fluid_meter::{create_fluid_meter, create_fluid_meter_at},
};

#[test(tokio::test)]
async fn alert_success() {
    let storage = PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await;

    // Fluid meter with constant flow alert
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

    // Fluid meter with not reporting alert and no measurements at all
    let fm3 = create_fluid_meter_at(Utc::now().naive_utc() - Duration::hours(25)).await;

    // Fluid meter with not reporting alert
    let fm4 = create_fluid_meter_at(Utc::now().naive_utc() - Duration::hours(25)).await;
    let m = Measurement {
        id: Uuid::new_v4().to_string(),
        device_id: fm4.id.clone(),
        measurement: "1".to_string(),
        recorded_at: Utc::now().naive_utc() - Duration::hours(25),
    };
    let _ = storage.save_measurement(&m).await;

    // Expect alert email submission
    let mut mail_helper_mock = MockMailHelper::new();
    let alerts = vec![FluidMeterAlerts {
        meter: fm.clone(),
        alerts: vec![Alert {
            alert_type: AlertType::ConstantFlow,
        }],
    }];
    let alerts2 = vec![FluidMeterAlerts {
        meter: fm3.clone(),
        alerts: vec![Alert {
            alert_type: AlertType::NotReporting,
        }],
    }];
    let alerts3 = vec![FluidMeterAlerts {
        meter: fm4.clone(),
        alerts: vec![Alert {
            alert_type: AlertType::NotReporting,
        }],
    }];
    mail_helper_mock
        .expect_alerts()
        .with(always(), always(), eq(alerts))
        .return_const(true);
    mail_helper_mock
        .expect_alerts()
        .with(always(), always(), eq(alerts2))
        .return_const(true);
    mail_helper_mock
        .expect_alerts()
        .with(always(), always(), eq(alerts3))
        .return_const(true);

    let app = create_app_mail_helper(Arc::new(mail_helper_mock)).await;

    let response = app
        .clone()
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

    // Expect rate limit
    let response = app
        .clone()
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
