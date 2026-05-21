use serde::{Deserialize, Serialize};
use std::fmt;
use strum::{AsRefStr, EnumIter, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
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
