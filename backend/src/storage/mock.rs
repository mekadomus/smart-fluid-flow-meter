use crate::{
    api::{
        common::{PaginatedRequest, PaginatedResponse},
        email_verification::EmailVerification,
        fluid_meter::{FluidMeter, FluidMetersInput},
        measurement::Measurement,
        metadata::Metadata,
        user::{NewPasswordInput, PasswordRecovery, SessionToken, User},
    },
    helper::mail::MailHelper,
    settings::settings::Settings,
    storage::{error::Error, FluidMeterStorage, MeasurementStorage, MetadataStorage, UserStorage},
};

use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::mock;
use std::sync::Arc;

mock! {
    pub Storage {}

    #[async_trait]
    impl crate::storage::Storage for Storage {}

    #[async_trait]
    impl FluidMeterStorage for Storage {
        async fn activate_fluid_meter(&self, meter_id: &str) -> Result<(), Error>;
        async fn deactivate_fluid_meter(&self, meter_id: &str) -> Result<(), Error>;
        async fn delete_fluid_meter(&self, id: &str) -> Result<(), Error>;
        async fn get_active_fluid_meters(
            &self,
            options: &PaginatedRequest,
        ) -> Result<PaginatedResponse<FluidMeter>, Error>;
        async fn get_fluid_meters(
            &self,
            user: &str,
            filters: &FluidMetersInput,
        ) -> Result<Vec<FluidMeter>, Error>;
        async fn get_fluid_meter_by_id(&self, id: &str) -> Result<Option<FluidMeter>, Error>;
        async fn insert_fluid_meter(&self, fluid_meter: &FluidMeter) -> Result<FluidMeter, Error>;
        async fn is_fluid_meter_owner(
            &self,
            fluid_meter_id: &str,
            account_id: &str,
        ) -> Result<bool, Error>;
    }

    #[async_trait]
    impl MeasurementStorage for Storage {
        async fn save_measurement(&self, measurement: &Measurement) -> Result<Measurement, Error>;
        async fn get_measurements(
            &self,
            device_id: String,
            from: NaiveDateTime,
            to: NaiveDateTime,
            num_records: u32,
        ) -> Result<Vec<Measurement>, Error>;
    }

    #[async_trait]
    impl MetadataStorage for Storage {
        async fn get_metadata(&self, key: &str) -> Result<Option<Metadata>, Error>;
        async fn save_metadata(&self, key: &str, value: &str) -> Result<Metadata, Error>;
    }

    #[async_trait]
    impl UserStorage for Storage {
        async fn email_verification_by_id(&self, id: &str) -> Result<Option<EmailVerification>, Error>;
        async fn insert_user(&self, user: &User) -> Result<User, Error>;
        async fn log_in(&self, id: &str) -> Result<SessionToken, Error>;
        async fn log_out(&self, token: &str) -> Result<(), Error>;
        async fn new_password(&self, input: &NewPasswordInput) -> Result<(), Error>;
        async fn password_recovery(
            &self,
            user: &User,
            settings: Arc<Settings>,
            mail_helper: Arc<dyn MailHelper>,
        ) -> Result<(), Error>;
        async fn password_recovery_by_user(
            &self,
            user_id: &str,
        ) -> Result<Option<PasswordRecovery>, Error>;
        async fn sign_up_user(
            &self,
            user: User,
            settings: Arc<Settings>,
            mail_helper: Arc<dyn MailHelper>,
        ) -> Result<User, Error>;
        async fn user_by_id(&self, id: &str) -> Result<Option<User>, Error>;
        async fn user_by_token(&self, token: &str) -> Result<Option<User>, Error>;
        async fn verify_email(&self, token: &str) -> Result<User, Error>;
    }
}
