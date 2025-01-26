use axum::{
    body::Body,
    http,
    http::{Request, StatusCode},
    Router,
};
use chrono::Local;
use http_body_util::BodyExt;
use mockall::predicate::always;
use std::sync::Arc;
use tower::util::ServiceExt;
use uuid::Uuid;

use smart_fluid_flow_meter_backend::{
    api::{
        common::PaginatedResponse,
        fluid_meter::{CreateFluidMeterInput, FluidMeter, FluidMeterStatus},
        user::{User, UserAuthProvider},
    },
    helper::{mail::MockMailHelper, user::MockUserHelper},
    middleware::auth::MockAuthorizer,
    settings::settings::Settings,
    storage::{firestore::FirestoreStorage, Storage},
};

async fn create_app_with_user(user_id: u16) -> (Router, Arc<dyn Storage>) {
    let settings = Arc::new(Settings::from_file(
        "/smart-fluid-flow-meter/tests/config/default.yaml",
    ));
    let storage = Arc::new(FirestoreStorage::new("dummy-id", "db-id").await);
    let mut authorizer = MockAuthorizer::new();
    authorizer
        .expect_authorize()
        .with(always(), always())
        .returning(move |_, request| {
            let user = User {
                id: user_id.to_string(),
                provider: UserAuthProvider::Password,
                name: "Carlos".to_string(),
                email: "carlos@example.com".to_string(),
                password: None,
                email_verified_at: None,
                recorded_at: Local::now(),
            };
            request.extensions_mut().insert(user);

            return Ok(());
        });
    let user_helper = Arc::new(MockUserHelper::new());
    let mail_helper = Arc::new(MockMailHelper::new());

    (
        smart_fluid_flow_meter_backend::app(
            Arc::new(authorizer),
            mail_helper,
            settings,
            storage.clone(),
            user_helper,
        )
        .await,
        storage,
    )
}

#[tokio::test]
async fn get_fluid_meters_empty() {
    let (app, _) = create_app_with_user(9999).await;
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
    let (app, storage) = create_app_with_user(user_id).await;

    let fluid_meter_1 = FluidMeter {
        id: user_id.to_string(),
        name: "kitchen".to_string(),
        owner_id: user_id.to_string(),
        status: FluidMeterStatus::Active,
        recorded_at: Local::now(),
    };
    let mut fluid_meter_2 = fluid_meter_1.clone();
    fluid_meter_2.id = (user_id + 1).to_string();
    fluid_meter_2.name = "bathroom".to_string();
    let mut fluid_meter_3 = fluid_meter_1.clone();
    fluid_meter_3.id = (user_id + 2).to_string();
    fluid_meter_3.name = "garage".to_string();
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
    let (app, _) = create_app_with_user(user_id).await;

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
    assert!(Local::now().timestamp_nanos_opt() > resp.recorded_at.timestamp_nanos_opt());
    assert_eq!(resp.owner_id, user_id.to_string());
    assert_eq!(resp.status, FluidMeterStatus::Active);
    assert!(Uuid::try_parse(&resp.id).is_ok());
}
