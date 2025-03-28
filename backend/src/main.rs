use smart_fluid_flow_meter_backend::{
    helper::{alert::DefaultAlertHelper, mail::DefaultMailHelper, user::DefaultUserHelper},
    middleware::auth::DefaultAuthorizer,
    settings::settings::Settings,
    storage::{postgres::PostgresStorage, Storage},
};

use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() {
    let alert_helper = Arc::new(DefaultAlertHelper {});
    let authorizer = Arc::new(DefaultAuthorizer {});
    let mail_helper = Arc::new(DefaultMailHelper {});
    let settings = Arc::new(Settings::new());
    let user_helper = Arc::new(DefaultUserHelper {});

    let storage: Arc<dyn Storage> =
        Arc::new(PostgresStorage::new(&settings.database.postgres.connection_string).await);

    let app = smart_fluid_flow_meter_backend::app(
        alert_helper,
        authorizer,
        mail_helper,
        settings.clone(),
        storage,
        user_helper,
    )
    .await;

    let listener = TcpListener::bind(format!("0.0.0.0:{}", settings.service.port))
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
