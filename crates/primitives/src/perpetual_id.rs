use std::fmt;
use std::str::FromStr;

use crate::CHAIN_SEPARATOR;
use crate::perpetual_provider::PerpetualProvider;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PerpetualId {
    pub provider: PerpetualProvider,
    pub symbol: String,
}

crate::impl_string_serde!(PerpetualId);

impl PerpetualId {
    pub fn new(provider: PerpetualProvider, symbol: &str) -> Self {
        Self {
            provider,
            symbol: symbol.to_string(),
        }
    }

    pub fn id(&self) -> String {
        self.to_string()
    }

    pub fn from_id(id: &str) -> Option<Self> {
        id.parse().ok()
    }
}

impl fmt::Display for PerpetualId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{CHAIN_SEPARATOR}{}", self.provider.as_ref(), self.symbol)
    }
}

impl FromStr for PerpetualId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (provider_str, symbol) = s
            .split_once(CHAIN_SEPARATOR)
            .ok_or_else(|| format!("invalid perpetual identifier format: expected 2 parts separated by '{CHAIN_SEPARATOR}', got: {s}"))?;
        let provider: PerpetualProvider = provider_str.parse().map_err(|_| format!("invalid perpetual provider: {provider_str}"))?;
        Ok(Self {
            provider,
            symbol: symbol.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_round_trip() {
        let id = PerpetualId::new(PerpetualProvider::Hypercore, "BTC");
        assert_eq!(id.id(), "hypercore_BTC");
        assert_eq!(PerpetualId::from_id("hypercore_BTC"), Some(id));
    }

    #[test]
    fn test_from_id_invalid() {
        assert!(PerpetualId::from_id("invalid").is_none());
        assert!(PerpetualId::from_id("unknown_BTC").is_none());
    }

    #[test]
    fn test_serde() {
        let id = PerpetualId::new(PerpetualProvider::Hypercore, "ETH");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"hypercore_ETH\"");
        let parsed: PerpetualId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }
}
