use crate::{
    api::{
        alert::{Alert, AlertType},
        fluid_meter::{FluidMeter, FluidMeterAlerts, FluidMeterStatus::Active},
        measurement::Measurement,
    },
    error::app_error::{internal_error, AppError},
    storage::Storage,
};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use mockall::automock;
use std::sync::Arc;
use tracing::error;

#[automock]
#[async_trait]
pub trait AlertHelper: Send + Sync {
    /// Returns all alerts for the given fluid meter
    async fn get_alerts(
        &self,
        storage: Arc<dyn Storage>,
        fluid_meter: &FluidMeter,
    ) -> Result<FluidMeterAlerts, AppError>;
    /// Returns true if the meter has had non-stop flow for a threshold
    fn has_constant_flow(&self, measurements: &Vec<Measurement>) -> bool;
    /// Returns true if the meter hasn't reported measuments for a threshold
    fn isnt_reporting(&self, fluid_meter: &FluidMeter, measurements: &Vec<Measurement>) -> bool;
}

pub const CONSTANT_FLOW_THRESHOLD: &'static usize = &5;
pub const MEASUREMENTS_PAGE_SIZE: &'static u8 = &10;
pub const NO_REPORTS_THRESHOLD: &'static Duration = &Duration::days(1);

pub struct DefaultAlertHelper;

#[async_trait]
impl AlertHelper for DefaultAlertHelper {
    async fn get_alerts(
        &self,
        storage: Arc<dyn Storage>,
        fluid_meter: &FluidMeter,
    ) -> Result<FluidMeterAlerts, AppError> {
        let mut result = FluidMeterAlerts {
            meter: fluid_meter.clone(),
            alerts: vec![],
        };

        let since = Utc::now().naive_utc() - Duration::hours(2);
        let measurements = match storage
            .get_measurements(
                fluid_meter.id.clone(),
                since,
                *MEASUREMENTS_PAGE_SIZE as u32,
            )
            .await
        {
            Ok(m) => m,
            Err(e) => {
                error!(
                    "Failed to get measurements for meter {}. Error: {}",
                    &fluid_meter.id, e
                );
                return internal_error();
            }
        };

        if self.has_constant_flow(&measurements) {
            result.alerts.push(Alert {
                alert_type: AlertType::ConstantFlow,
            });
        }

        if self.isnt_reporting(&fluid_meter, &measurements) {
            result.alerts.push(Alert {
                alert_type: AlertType::NotReporting,
            });
        }

        return Ok(result);
    }

    fn has_constant_flow(&self, measurements: &Vec<Measurement>) -> bool {
        if measurements.len() < *CONSTANT_FLOW_THRESHOLD {
            return false;
        }

        for n in 0..*CONSTANT_FLOW_THRESHOLD {
            if measurements[n].measurement.parse::<f32>().unwrap() == 0.0 {
                return false;
            }
        }

        return true;
    }

    fn isnt_reporting(&self, fluid_meter: &FluidMeter, measurements: &Vec<Measurement>) -> bool {
        let now = Utc::now().naive_utc();
        if fluid_meter.status != Active || now - fluid_meter.updated_at < *NO_REPORTS_THRESHOLD {
            return false;
        }

        if measurements.len() == 0 || now - measurements[0].recorded_at > *NO_REPORTS_THRESHOLD {
            return true;
        }

        return false;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        api::{
            alert::{Alert, AlertType},
            fluid_meter::{
                FluidMeter, FluidMeterAlerts,
                FluidMeterStatus::{Active, Inactive},
            },
        },
        helper::alert::{AlertHelper, DefaultAlertHelper, Measurement},
        storage::mock::MockStorage,
    };

    use chrono::{Duration, Utc};
    use mockall::predicate::{always, eq};
    use std::sync::Arc;

    #[test]
    fn isnt_reporting_not_active() {
        let fm = FluidMeter {
            id: "some_id".to_string(),
            owner_id: "some_owner_id".to_string(),
            name: "name".to_string(),
            status: Inactive,
            recorded_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        let helper = DefaultAlertHelper {};
        let v: Vec<Measurement> = vec![];
        assert!(!helper.isnt_reporting(&fm, &v));
    }

    #[test]
    fn isnt_reporting_updated_at() {
        let fm = FluidMeter {
            id: "some_id".to_string(),
            owner_id: "some_owner_id".to_string(),
            name: "name".to_string(),
            status: Active,
            recorded_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        let helper = DefaultAlertHelper {};
        let v: Vec<Measurement> = vec![];
        assert!(!helper.isnt_reporting(&fm, &v));
    }

    #[test]
    fn isnt_reporting_last_measurement() {
        let fm = FluidMeter {
            id: "some_id".to_string(),
            owner_id: "some_owner_id".to_string(),
            name: "name".to_string(),
            status: Active,
            recorded_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc() - Duration::hours(25),
        };

        let helper = DefaultAlertHelper {};
        let v: Vec<Measurement> = vec![Measurement {
            id: "id".to_string(),
            measurement: "0.0".to_string(),
            device_id: "some_id".to_string(),
            recorded_at: Utc::now().naive_utc(),
        }];
        assert!(!helper.isnt_reporting(&fm, &v));
    }

    #[test]
    fn isnt_reporting_alerting() {
        let fm = FluidMeter {
            id: "some_id".to_string(),
            owner_id: "some_owner_id".to_string(),
            name: "name".to_string(),
            status: Active,
            recorded_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc() - Duration::hours(25),
        };

        let helper = DefaultAlertHelper {};
        let v: Vec<Measurement> = vec![Measurement {
            id: "id".to_string(),
            measurement: "0.0".to_string(),
            device_id: "some_id".to_string(),
            recorded_at: Utc::now().naive_utc() - Duration::hours(25),
        }];
        assert!(helper.isnt_reporting(&fm, &v));
    }

    #[test]
    fn has_constant_flow_no_measurements() {
        let helper = DefaultAlertHelper {};
        let v: Vec<Measurement> = vec![];
        assert!(!helper.has_constant_flow(&v));
    }

    #[test]
    fn has_constant_flow_not_alerting() {
        let helper = DefaultAlertHelper {};
        let v: Vec<Measurement> = vec![
            Measurement {
                id: "id".to_string(),
                measurement: "1.0".to_string(),
                device_id: "some_id".to_string(),
                recorded_at: Utc::now().naive_utc() - Duration::hours(25),
            },
            Measurement {
                id: "id".to_string(),
                measurement: "0.0".to_string(),
                device_id: "some_id".to_string(),
                recorded_at: Utc::now().naive_utc() - Duration::hours(25),
            },
            Measurement {
                id: "id".to_string(),
                measurement: "1.0".to_string(),
                device_id: "some_id".to_string(),
                recorded_at: Utc::now().naive_utc() - Duration::hours(25),
            },
            Measurement {
                id: "id".to_string(),
                measurement: "1.0".to_string(),
                device_id: "some_id".to_string(),
                recorded_at: Utc::now().naive_utc() - Duration::hours(25),
            },
            Measurement {
                id: "id".to_string(),
                measurement: "1.0".to_string(),
                device_id: "some_id".to_string(),
                recorded_at: Utc::now().naive_utc() - Duration::hours(25),
            },
        ];
        assert!(!helper.has_constant_flow(&v));
    }

    #[test]
    fn has_constant_flow_alerting() {
        let helper = DefaultAlertHelper {};
        let v: Vec<Measurement> = vec![
            Measurement {
                id: "id".to_string(),
                measurement: "1.0".to_string(),
                device_id: "some_id".to_string(),
                recorded_at: Utc::now().naive_utc() - Duration::hours(25),
            },
            Measurement {
                id: "id".to_string(),
                measurement: "1.0".to_string(),
                device_id: "some_id".to_string(),
                recorded_at: Utc::now().naive_utc() - Duration::hours(25),
            },
            Measurement {
                id: "id".to_string(),
                measurement: "1.0".to_string(),
                device_id: "some_id".to_string(),
                recorded_at: Utc::now().naive_utc() - Duration::hours(25),
            },
            Measurement {
                id: "id".to_string(),
                measurement: "1.0".to_string(),
                device_id: "some_id".to_string(),
                recorded_at: Utc::now().naive_utc() - Duration::hours(25),
            },
            Measurement {
                id: "id".to_string(),
                measurement: "1.0".to_string(),
                device_id: "some_id".to_string(),
                recorded_at: Utc::now().naive_utc() - Duration::hours(25),
            },
        ];
        assert!(helper.has_constant_flow(&v));
    }

    #[tokio::test]
    async fn get_alerts_success() {
        let device_id = "dev_id";
        let fm = FluidMeter {
            id: device_id.to_string(),
            owner_id: "some_owner_id".to_string(),
            name: "name".to_string(),
            status: Active,
            recorded_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc() - Duration::hours(25),
        };

        let measurements = vec![Measurement {
            id: "id".to_string(),
            measurement: "1.0".to_string(),
            device_id: "some_id".to_string(),
            recorded_at: Utc::now().naive_utc() - Duration::hours(25),
        }];
        let mut storage = MockStorage::new();
        storage
            .expect_get_measurements()
            .with(eq(device_id.to_string()), always(), eq(10))
            .return_const(Ok(measurements));

        let expected = FluidMeterAlerts {
            meter: fm.clone(),
            alerts: vec![Alert {
                alert_type: AlertType::NotReporting,
            }],
        };
        let helper = DefaultAlertHelper {};
        assert_eq!(
            expected,
            helper.get_alerts(Arc::new(storage), &fm).await.unwrap()
        );
    }
}
