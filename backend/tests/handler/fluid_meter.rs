use axum::{
    body::Body,
    http,
    http::{Request, StatusCode},
};
use chrono::Utc;
use http_body_util::BodyExt;
use mockall::predicate::{always, eq};
use std::sync::Arc;
use tower::util::ServiceExt;
use uuid::Uuid;

use smart_fluid_flow_meter_backend::{
    api::{
        common::PaginatedResponse,
        fluid_meter::{CreateFluidMeterInput, FluidMeter, FluidMeterStatus},
    },
    helper::user::MockUserHelper,
    storage::{postgres::PostgresStorage, FluidMeterStorage},
};

use crate::helper::app::{create_app_with_user, create_app_with_user_user_helper};

#[tokio::test]
async fn get_fluid_meters_empty() {
    let app = create_app_with_user(9999).await;
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/v1/fluid-meter")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: PaginatedResponse<FluidMeter> = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp.items.len(), 0);
}

#[tokio::test]
async fn get_fluid_meters_filters() {
    let user_id = 158;
    let app = create_app_with_user(user_id).await;
    let storage =
        Arc::new(PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await);

    let fluid_meter_1 = FluidMeter {
        id: user_id.to_string(),
        name: "kitchen".to_string(),
        owner_id: user_id.to_string(),
        status: FluidMeterStatus::Active,
        recorded_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };
    let mut fluid_meter_2 = fluid_meter_1.clone();
    fluid_meter_2.id = (user_id + 1).to_string();
    fluid_meter_2.name = "bathroom".to_string();
    let mut fluid_meter_3 = fluid_meter_1.clone();
    fluid_meter_3.id = (user_id + 2).to_string();
    fluid_meter_3.name = "garage".to_string();
    fluid_meter_3.status = FluidMeterStatus::Inactive;
    assert!(storage.insert_fluid_meter(&fluid_meter_1).await.is_ok());
    assert!(storage.insert_fluid_meter(&fluid_meter_2).await.is_ok());
    assert!(storage.insert_fluid_meter(&fluid_meter_3).await.is_ok());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/v1/fluid-meter")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: PaginatedResponse<FluidMeter> = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp.items.len(), 3);
    assert_eq!(resp.items[0].id, fluid_meter_1.id);
    assert_eq!(resp.items[2].id, fluid_meter_3.id);
    assert!(!resp.pagination.has_more);
    assert!(!resp.pagination.has_less);

    // Only active meters
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/v1/fluid-meter?status=Active")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: PaginatedResponse<FluidMeter> = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp.items.len(), 2);
    assert!(!resp.pagination.has_more);
    assert!(!resp.pagination.has_less);

    // First page sorting by id in descending order
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/v1/fluid-meter?page_size=1&sort_direction=Desc&sort=Id")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: PaginatedResponse<FluidMeter> = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.items[0].id, fluid_meter_3.id);
    assert!(resp.pagination.has_more);
    assert!(!resp.pagination.has_less);

    // Second page sorting by name in ascending order
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(
                    "/v1/fluid-meter?page_size=1&sort_direction=Asc&sort=Name&page_cursor=bathroom",
                )
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: PaginatedResponse<FluidMeter> = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.items[0].name, "garage");
    assert!(resp.pagination.has_more);
    assert!(resp.pagination.has_less);
}

#[tokio::test]
async fn create_fluid_meter() {
    let user_id = 500;
    let app = create_app_with_user(user_id).await;

    let name = "kitchen";
    let input = CreateFluidMeterInput {
        name: name.to_string(),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/fluid-meter")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: FluidMeter = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp.name, name);
    assert!(Utc::now().naive_utc() > resp.recorded_at);
    assert_eq!(resp.owner_id, user_id.to_string());
    assert_eq!(resp.status, FluidMeterStatus::Active);
    assert!(Uuid::try_parse(&resp.id).is_ok());
}

#[tokio::test]
async fn get_fluid_meter_success() {
    let user_id = 500;
    let fm_id = Uuid::new_v4().to_string();

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_owns_fluid_meter()
        .with(always(), eq(user_id.to_string()), eq(fm_id.to_string()))
        .returning(|_, _, _| Ok(true));

    let app = create_app_with_user_user_helper(user_id, Arc::new(user_helper_mock)).await;
    let storage =
        Arc::new(PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await);

    let fm = FluidMeter {
        id: fm_id.to_string(),
        name: "kitchen".to_string(),
        owner_id: user_id.to_string(),
        status: FluidMeterStatus::Active,
        recorded_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };
    assert!(storage.insert_fluid_meter(&fm).await.is_ok());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/v1/fluid-meter/{}", &fm.id))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: FluidMeter = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp.name, fm.name);
    assert!(Utc::now().naive_utc() > resp.recorded_at);
    assert_eq!(resp.owner_id, user_id.to_string());
    assert_eq!(resp.status, FluidMeterStatus::Active);
    assert!(Uuid::try_parse(&resp.id).is_ok());
}

#[tokio::test]
async fn get_fluid_meter_no_owner() {
    let user_id = 555;
    let _ = create_app_with_user(555).await;

    let fm_id = Uuid::new_v4();
    let req_user_id = 554;

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_owns_fluid_meter()
        .with(always(), eq(req_user_id.to_string()), eq(fm_id.to_string()))
        .returning(|_, _, _| Ok(false));

    let app = create_app_with_user_user_helper(req_user_id, Arc::new(user_helper_mock)).await;
    let storage =
        Arc::new(PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await);

    // The user for `app` is 554, but the fluid meter owner is 555
    let fm = FluidMeter {
        id: fm_id.to_string(),
        name: "kitchen".to_string(),
        owner_id: user_id.to_string(),
        status: FluidMeterStatus::Active,
        recorded_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };
    assert!(storage.insert_fluid_meter(&fm).await.is_ok());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/v1/fluid-meter/{}", &fm.id))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn activate_deactivate_fluid_meter() {
    let user_id = 321;
    let fm_id = Uuid::new_v4().to_string();

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_owns_fluid_meter()
        .with(always(), eq(user_id.to_string()), eq(fm_id.to_string()))
        .returning(|_, _, _| Ok(true));

    let app = create_app_with_user_user_helper(user_id, Arc::new(user_helper_mock)).await;
    let storage =
        Arc::new(PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await);

    let fm = FluidMeter {
        id: fm_id.to_string(),
        name: "kitchen".to_string(),
        owner_id: user_id.to_string(),
        status: FluidMeterStatus::Active,
        recorded_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };
    assert!(storage.insert_fluid_meter(&fm).await.is_ok());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/v1/fluid-meter/{}", &fm.id))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("/v1/fluid-meter/{}/deactivate", &fm.id))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        storage
            .get_fluid_meter_by_id(&fm_id)
            .await
            .unwrap()
            .unwrap()
            .status,
        FluidMeterStatus::Inactive
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("/v1/fluid-meter/{}/activate", &fm.id))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        storage
            .get_fluid_meter_by_id(&fm_id)
            .await
            .unwrap()
            .unwrap()
            .status,
        FluidMeterStatus::Active
    );
}

#[tokio::test]
async fn delete_fluid_meter() {
    let user_id = 322;
    let fm_id = Uuid::new_v4().to_string();

    // Mock UserHelper
    let mut user_helper_mock = MockUserHelper::new();
    user_helper_mock
        .expect_owns_fluid_meter()
        .with(always(), eq(user_id.to_string()), eq(fm_id.to_string()))
        .returning(|_, _, _| Ok(true));

    let app = create_app_with_user_user_helper(user_id, Arc::new(user_helper_mock)).await;
    let storage =
        Arc::new(PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await);

    let fm = FluidMeter {
        id: fm_id.to_string(),
        name: "kitchen".to_string(),
        owner_id: user_id.to_string(),
        status: FluidMeterStatus::Active,
        recorded_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };
    assert!(storage.insert_fluid_meter(&fm).await.is_ok());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::DELETE)
                .uri(format!("/v1/fluid-meter/{}", &fm.id))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        storage
            .get_fluid_meter_by_id(&fm_id)
            .await
            .unwrap()
            .unwrap()
            .status,
        FluidMeterStatus::Deleted
    );
}
