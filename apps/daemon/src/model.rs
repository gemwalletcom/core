use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, AsRefStr, EnumString, EnumIter, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum DaemonService {
    Alerter,
    Pricer,
    Fiat,
    FiatConsumer,
    Assets,
    Version,
    Transaction,
    Device,
    Search,
    Nft,
    Notifications,
    Scan,
    ConsumerSupport,
}

impl DaemonService {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}
