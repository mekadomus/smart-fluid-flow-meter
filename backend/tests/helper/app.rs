use axum::Router;
use chrono::Utc;
use mockall::predicate::always;
use std::sync::Arc;

use smart_fluid_flow_meter_backend::{
    api::user::{User, UserAuthProvider::Password},
    helper::{
        alert::DefaultAlertHelper,
        mail::{MailHelper, MockMailHelper},
        user::{MockUserHelper, UserHelper},
    },
    middleware::auth::{Authorizer, DefaultAuthorizer, MockAuthorizer},
    settings::settings::Settings,
    storage::{postgres::PostgresStorage, UserStorage},
};

pub async fn create_app_basic() -> Router {
    let authorizer = DefaultAuthorizer {};
    let mail_helper = MockMailHelper::new();
    let user_helper = MockUserHelper::new();
    return create_app(
        Arc::new(authorizer),
        Arc::new(mail_helper),
        Arc::new(user_helper),
    )
    .await;
}

pub async fn create_app_user_helper(user_helper: Arc<dyn UserHelper>) -> Router {
    let authorizer = DefaultAuthorizer {};
    let mail_helper = MockMailHelper::new();
    return create_app(
        Arc::new(authorizer),
        Arc::new(mail_helper),
        user_helper.clone(),
    )
    .await;
}

pub async fn create_app_mail_helper(mail_helper: Arc<dyn MailHelper>) -> Router {
    let authorizer = DefaultAuthorizer {};
    let user_helper = MockUserHelper::new();
    return create_app(
        Arc::new(authorizer),
        mail_helper.clone(),
        Arc::new(user_helper),
    )
    .await;
}

pub async fn create_app_with_user(user_id: u16) -> Router {
    let user_helper = MockUserHelper::new();
    create_app_with_user_user_helper(user_id, Arc::new(user_helper)).await
}

pub async fn create_app_with_user_user_helper(
    user_id: u16,
    user_helper: Arc<dyn UserHelper>,
) -> Router {
    let storage =
        Arc::new(PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await);

    let user = User {
        id: user_id.to_string(),
        provider: Password,
        name: "Carlos".to_string(),
        email: "a@b.c".to_string(),
        password: Some("hello".to_string()),
        email_verified_at: Some(Utc::now().naive_utc()),
        recorded_at: Utc::now().naive_utc(),
    };
    let _ = storage.insert_user(&user).await;

    let mut authorizer = MockAuthorizer::new();
    authorizer
        .expect_authorize()
        .with(always(), always())
        .returning(move |_, request| {
            let user = User {
                id: user_id.to_string(),
                provider: Password,
                name: "Carlos".to_string(),
                email: "carlos@example.com".to_string(),
                password: None,
                email_verified_at: None,
                recorded_at: Utc::now().naive_utc(),
            };
            request.extensions_mut().insert(user);

            return Ok(());
        });
    let mail_helper = Arc::new(MockMailHelper::new());

    create_app(Arc::new(authorizer), mail_helper, user_helper.clone()).await
}

pub async fn create_app(
    authorizer: Arc<dyn Authorizer>,
    mail_helper: Arc<dyn MailHelper>,
    user_helper: Arc<dyn UserHelper>,
) -> Router {
    let settings = Arc::new(Settings::from_file(
        "/smart-fluid-flow-meter/tests/config/default.yaml",
    ));
    let storage =
        Arc::new(PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await);
    return smart_fluid_flow_meter_backend::app(
        Arc::new(DefaultAlertHelper {}),
        authorizer,
        mail_helper,
        settings,
        storage.clone(),
        user_helper,
    )
    .await;
}
