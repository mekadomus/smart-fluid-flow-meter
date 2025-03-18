use chrono::{NaiveDateTime, Timelike, Utc};
use uuid::Uuid;

use smart_fluid_flow_meter_backend::{
    api::{
        fluid_meter::{FluidMeter, FluidMeterStatus::Active},
        user::{User, UserAuthProvider::Password},
    },
    storage::{postgres::PostgresStorage, FluidMeterStorage, UserStorage},
};

/// Creates a user and a fluid meter. The fluid meter will be created at the
/// given time
pub async fn create_fluid_meter_at(creation_time: NaiveDateTime) -> FluidMeter {
    let storage = PostgresStorage::new("postgresql://user:password@postgres/mekadomus").await;

    let user_name = Uuid::new_v4().to_string();
    let user_email = format!("{}@example.com", user_name);
    let user_id = format!("{}+password", user_email);
    let user = User {
        id: user_id.to_string(),
        provider: Password,
        name: user_name.to_string(),
        email: user_email.to_string(),
        password: Some(user_name.to_string()),
        email_verified_at: Some(Utc::now().naive_utc()),
        recorded_at: creation_time.clone(),
    };
    let fm_id = Uuid::new_v4().to_string();
    let at = creation_time
        .clone()
        .with_nanosecond(0)
        .expect("Couldn't format date for fluid meter");
    let fm = FluidMeter {
        id: fm_id.to_string(),
        owner_id: user_id.to_string(),
        name: fm_id.to_string(),
        status: Active,
        recorded_at: at,
        updated_at: at,
    };
    let _ = storage.insert_user(&user).await;
    let _ = storage.insert_fluid_meter(&fm).await;

    fm
}

/// Creates a user and a fluid meter associated to that user
pub async fn create_fluid_meter() -> FluidMeter {
    create_fluid_meter_at(Utc::now().naive_utc()).await
}
