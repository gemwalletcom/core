use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{AsRefStr, EnumString};

#[derive(Debug, Clone, AsRefStr, EnumString, EnumIter, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum DaemonService {
    Alerter,
    Pricer,
    Fiat,
    Assets,
    Version,
    Transaction,
    Device,
    Search,
    Nft,
}

impl DaemonService {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}
