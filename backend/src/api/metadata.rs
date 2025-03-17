use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, sqlx::FromRow, Debug)]
pub struct Metadata {
    pub key: String,
    pub value: String,
}
