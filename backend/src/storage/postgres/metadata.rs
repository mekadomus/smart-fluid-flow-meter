use async_trait::async_trait;
use tracing::error;

use crate::{
    api::metadata::Metadata,
    storage::{
        error::{undefined, Error},
        postgres::PostgresStorage,
        MetadataStorage,
    },
};

#[async_trait]
impl MetadataStorage for PostgresStorage {
    async fn get_metadata(&self, key: &str) -> Result<Option<Metadata>, Error> {
        match sqlx::query_as(
            r#"
            SELECT *
            FROM metadata
            WHERE key = $1
            "#,
        )
        .bind(key)
        .fetch_one(&self.pool)
        .await
        {
            Ok(m) => Ok(Some(m)),
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => {
                    error!("Error getting measurements for device. {}", e);
                    return undefined();
                }
            },
        }
    }

    async fn save_metadata(&self, key: &str, value: &str) -> Result<Metadata, Error> {
        match sqlx::query(
            r#"
            INSERT INTO metadata(key, value)
            VALUES($1, $2)
            ON CONFLICT(key)
            DO UPDATE SET value = $2
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(value)
        .execute(&self.pool)
        .await
        {
            Ok(_) => Ok(Metadata {
                key: key.to_string(),
                value: value.to_string(),
            }),
            Err(e) => {
                error!("Error saving metadata. {}:{} . Error: {}", key, value, e);
                return undefined();
            }
        }
    }
}
