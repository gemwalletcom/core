use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, EnumIter, PartialEq, Eq, Hash)]
#[strum(serialize_all = "camelCase")]
pub enum ConfigKey {
    ReferralPerIpDaily,
    ReferralPerIpWeekly,
}

impl ConfigKey {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn default_value(&self) -> &'static str {
        match self {
            Self::ReferralPerIpDaily => "3",
            Self::ReferralPerIpWeekly => "10",
        }
    }
}
