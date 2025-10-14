use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, Eq, AsRefStr, EnumString, EnumIter, Serialize, Deserialize)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PriceFeedProvider {
    Pyth,
    Jupiter,
}

impl PriceFeedProvider {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn feed_id(&self, id: &str) -> String {
        format!("{}_{}", self.as_ref(), id)
    }
}

#[derive(Debug, Clone)]
pub struct PriceFeedId {
    pub feed_id: String,
    pub provider: PriceFeedProvider,
}

impl PriceFeedId {
    pub fn new(provider: PriceFeedProvider, feed_id: String) -> Self {
        Self { feed_id, provider }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        let (provider_str, feed_id) = id.split_once('_')?;
        let provider: PriceFeedProvider = provider_str.parse().ok()?;
        Some(Self::new(provider, feed_id.to_string()))
    }

    pub fn get_id(&self) -> String {
        format!("{}_{}", self.provider.as_ref(), self.feed_id)
    }
}
