use smart_fluid_flow_meter_backend::{
    helper::{mail::DefaultMailHelper, user::DefaultUserHelper},
    middleware::auth::DefaultAuthorizer,
    settings::settings::Settings,
    storage::{firestore::FirestoreStorage, Storage},
};

use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_line_number(true)
        .init();

    let authorizer = Arc::new(DefaultAuthorizer {});
    let mail_helper = Arc::new(DefaultMailHelper {});
    let settings = Arc::new(Settings::new());
    let user_helper = Arc::new(DefaultUserHelper {});

    let storage: Arc<dyn Storage> = Arc::new(
        FirestoreStorage::new(
            &settings.database.firestore.project_id,
            &settings.database.firestore.database_id,
        )
        .await,
    );
    let app = smart_fluid_flow_meter_backend::app(
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
