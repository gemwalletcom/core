use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumIter, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum UsernameStatus {
    Unverified,
    Verified,
}

impl UsernameStatus {
    pub fn is_verified(&self) -> bool {
        match self {
            Self::Verified => true,
            Self::Unverified => false,
        }
    }
}
