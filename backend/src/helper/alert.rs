use crate::api::{
    fluid_meter::{FluidMeter, FluidMeterStatus::Active},
    measurement::Measurement,
};

use chrono::{Duration, Utc};
use mockall::automock;

pub const CONSTANT_THRESHOLD: &'static u8 = &4;

#[automock]
pub trait AlertHelper: Send + Sync {
    /// Returns true if the meter has had non-stop flow for a threshold
    fn has_constant_flow(&self, measurements: &Vec<Measurement>) -> bool;
    /// Returns true if the meter hasn't reported measuments for a threshold
    fn isnt_reporting(&self, fluid_meter: &FluidMeter, measurements: &Vec<Measurement>) -> bool;
}

pub const CONSTANT_FLOW_THRESHOLD: &'static usize = &5;
pub const NO_REPORTS_THRESHOLD: &'static Duration = &Duration::days(1);

pub struct DefaultAlertHelper;

impl AlertHelper for DefaultAlertHelper {
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
