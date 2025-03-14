use axum::Router;
use std::sync::Arc;

use smart_fluid_flow_meter_backend::{
    helper::{
        alert::DefaultAlertHelper,
        mail::{MailHelper, MockMailHelper},
        user::{MockUserHelper, UserHelper},
    },
    middleware::auth::DefaultAuthorizer,
    settings::settings::Settings,
    storage::postgres::PostgresStorage,
};

pub async fn create_app_basic() -> Router {
    let mail_helper = MockMailHelper::new();
    let user_helper = MockUserHelper::new();
    return create_app(Arc::new(mail_helper), Arc::new(user_helper)).await;
}

pub async fn create_app_user_helper(user_helper: Arc<dyn UserHelper>) -> Router {
    let mail_helper = MockMailHelper::new();
    return create_app(Arc::new(mail_helper), user_helper.clone()).await;
}

pub async fn create_app_mail_helper(mail_helper: Arc<dyn MailHelper>) -> Router {
    let user_helper = MockUserHelper::new();
    return create_app(mail_helper.clone(), Arc::new(user_helper)).await;
}

pub async fn create_app(
    mail_helper: Arc<dyn MailHelper>,
    user_helper: Arc<dyn UserHelper>,
) -> Router {
    let settings = Arc::new(Settings::from_file(
        "/smart-fluid-flow-meter/tests/config/default.yaml",
    ));
    let storage =
        Arc::new(PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await);
    let authorizer = Arc::new(DefaultAuthorizer {});
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
