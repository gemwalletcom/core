use serde::{Deserialize, Serialize};
use std::fmt;
use strum::EnumIter;
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PerpetualProvider {
    Hypercore,
}

impl fmt::Display for PerpetualProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PerpetualProvider::Hypercore => write!(f, "hypercore"),
        }
    }
}