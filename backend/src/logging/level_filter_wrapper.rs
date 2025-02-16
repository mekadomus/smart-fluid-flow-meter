use serde::{Deserialize, Deserializer};
use std::str::FromStr;
use tracing::level_filters::LevelFilter;

#[derive(Debug)]
pub struct LevelFilterWrapper(pub LevelFilter);

impl<'de> Deserialize<'de> for LevelFilterWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        LevelFilter::from_str(&s)
            .map(LevelFilterWrapper)
            .map_err(serde::de::Error::custom)
    }
}
