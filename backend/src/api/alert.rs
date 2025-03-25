use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AlertType {
    ConstantFlow,
    NotReporting,
}

impl fmt::Display for AlertType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AlertType::ConstantFlow => "ConstantFlow",
                AlertType::NotReporting => "NotReporting",
            }
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Alert {
    pub alert_type: AlertType,
}
