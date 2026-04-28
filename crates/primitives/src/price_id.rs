use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::str::FromStr;

use crate::{CHAIN_SEPARATOR, PriceProvider};

/// The resolved (provider, provider_price_id) pair used to key prices, charts and any other
/// provider-scoped data. `id()` produces the synthetic `prices.id` used across the schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PriceId {
    pub provider: PriceProvider,
    pub provider_price_id: String,
}

impl PriceId {
    pub fn new(provider: PriceProvider, provider_price_id: String) -> Self {
        Self { provider, provider_price_id }
    }

    pub fn id(&self) -> String {
        self.to_string()
    }

    pub fn id_for(provider: PriceProvider, provider_price_id: &str) -> String {
        format!("{provider}{CHAIN_SEPARATOR}{provider_price_id}")
    }
}

impl fmt::Display for PriceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Self::id_for(self.provider, &self.provider_price_id))
    }
}

impl FromStr for PriceId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (provider, provider_price_id) = s.split_once(CHAIN_SEPARATOR).ok_or_else(|| format!("Invalid price_id: {s}"))?;
        if provider_price_id.is_empty() {
            return Err(format!("Invalid price_id: {s}"));
        }
        let provider = provider.parse().map_err(|_| format!("Unknown provider: {provider}"))?;
        Ok(Self {
            provider,
            provider_price_id: provider_price_id.to_string(),
        })
    }
}

impl Serialize for PriceId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PriceId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}
