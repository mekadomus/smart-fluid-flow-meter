use crate::api::measurement::Measurement;

use mockall::automock;

pub const CONSTANT_THRESHOLD: &'static u8 = &4;

#[automock]
pub trait AlertHelper: Send + Sync {
    /// Returns true if the meter has had non-stop flow for a threshold
    fn has_constant_flow(&self, measurements: &Vec<Measurement>) -> bool;
}

pub struct DefaultAlertHelper;

impl AlertHelper for DefaultAlertHelper {
    fn has_constant_flow(&self, measurements: &Vec<Measurement>) -> bool {
        if measurements.len() < 5 {
            return false;
        }

        for n in 0..5 {
            if measurements[n].measurement.parse::<f32>().unwrap() == 0.0 {
                return false;
            }
        }

        return true;
    }
}
