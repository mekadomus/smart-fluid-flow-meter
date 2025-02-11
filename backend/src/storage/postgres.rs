pub mod fluid_meter;
pub mod measurement;
pub mod user;

use sqlx::{
    postgres::{PgPoolOptions, Postgres},
    Pool,
};
use tracing::info;

use crate::storage::Storage;

pub const UNIQUE_VIOLATION: &str = "23505";

#[derive(Clone)]
pub struct PostgresStorage {
    pool: Pool<Postgres>,
}

impl PostgresStorage {
    pub async fn new(connection_string: &str) -> PostgresStorage {
        let pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await
        {
            Ok(pool) => pool,
            Err(err) => panic!(
                "Unable to create Postgres connection pool using {}. Error: {}",
                connection_string, err
            ),
        };

        match sqlx::migrate!("assets/db-migrations").run(&pool).await {
            Ok(()) => info!("DB migrations ran successfully"),
            Err(err) => panic!("Unable to run Postgres migrations. Error: {}", err),
        };

        return PostgresStorage { pool };
    }
}

impl Storage for PostgresStorage {}
