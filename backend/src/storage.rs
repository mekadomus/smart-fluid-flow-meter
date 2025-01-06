pub mod error;
pub mod firestore;

use crate::{
    api::{measurement::Measurement, user::User},
    helper::mail::MailHelper,
    settings::settings::Settings,
    storage::error::Error,
};

use async_trait::async_trait;
use chrono::{DateTime, Local};
use std::sync::Arc;

#[async_trait]
pub trait Storage: Send + Sync {
    // ----- Measurement ----- //
    async fn save_measurement(&self, measurement: Measurement) -> Result<Measurement, Error>;
    async fn get_measurements(
        &self,
        device_id: String,
        since: DateTime<Local>,
        num_records: u32,
    ) -> Result<Vec<Measurement>, Error>;

    // ----- User ----- //
    async fn sign_up_user(
        &self,
        user: User,
        settings: Arc<Settings>,
        mail_helper: Arc<dyn MailHelper>,
    ) -> Result<User, Error>;
    async fn user_by_id(&self, id: &str) -> Result<Option<User>, Error>;
}
