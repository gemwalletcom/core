use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

use crate::chain::Chain;
use crate::wallet_type::WalletType;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WalletId {
    Multicoin(String),
    Single(Chain, String),
    PrivateKey(Chain, String),
    View(Chain, String),
}

impl Serialize for WalletId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.id())
    }
}

impl<'de> Deserialize<'de> for WalletId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        WalletId::from_id(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid wallet identifier: {}", s)))
    }
}

impl WalletId {
    pub fn id(&self) -> String {
        self.to_string()
    }

    pub fn from_id(id: &str) -> Option<Self> {
        id.parse().ok()
    }

    pub fn wallet_type(&self) -> WalletType {
        match self {
            WalletId::Multicoin(_) => WalletType::Multicoin,
            WalletId::Single(_, _) => WalletType::Single,
            WalletId::PrivateKey(_, _) => WalletType::PrivateKey,
            WalletId::View(_, _) => WalletType::View,
        }
    }

    pub fn address(&self) -> &str {
        match self {
            WalletId::Multicoin(address)
            | WalletId::Single(_, address)
            | WalletId::PrivateKey(_, address)
            | WalletId::View(_, address) => address,
        }
    }

    pub fn chain(&self) -> Option<Chain> {
        match self {
            WalletId::Multicoin(_) => None,
            WalletId::Single(chain, _) | WalletId::PrivateKey(chain, _) | WalletId::View(chain, _) => Some(*chain),
        }
    }
}

impl fmt::Display for WalletId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletId::Multicoin(address) => write!(f, "{}_{}", WalletType::Multicoin.as_ref(), address),
            WalletId::Single(chain, address) | WalletId::PrivateKey(chain, address) | WalletId::View(chain, address) => {
                write!(f, "{}_{}_{}", self.wallet_type().as_ref(), chain.as_ref(), address)
            }
        }
    }
}

impl FromStr for WalletId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err(format!("invalid wallet identifier format: expected at least 2 parts separated by '_', got: {}", s));
        }
        let wallet_type: WalletType = parts[0].parse().map_err(|_| format!("invalid wallet type: {}", parts[0]))?;
        let rest = parts[1];

        match wallet_type {
            WalletType::Multicoin => Ok(WalletId::Multicoin(rest.to_string())),
            WalletType::Single | WalletType::PrivateKey | WalletType::View => {
                let chain_parts: Vec<&str> = rest.splitn(2, '_').collect();
                if chain_parts.len() != 2 {
                    return Err(format!("invalid wallet identifier format for {}: expected 3 parts, got: {}", wallet_type.as_ref(), s));
                }
                let chain: Chain = chain_parts[0].parse().map_err(|_| format!("invalid chain: {}", chain_parts[0]))?;
                let address = chain_parts[1].to_string();
                match wallet_type {
                    WalletType::Single => Ok(WalletId::Single(chain, address)),
                    WalletType::PrivateKey => Ok(WalletId::PrivateKey(chain, address)),
                    WalletType::View => Ok(WalletId::View(chain, address)),
                    _ => unreachable!(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_identifier_id() {
        assert_eq!(WalletId::Multicoin("0x123".to_string()).id(), "multicoin_0x123");
        assert_eq!(WalletId::Single(Chain::Ethereum, "0x456".to_string()).id(), "single_ethereum_0x456");
        assert_eq!(WalletId::PrivateKey(Chain::Bitcoin, "bc1".to_string()).id(), "privateKey_bitcoin_bc1");
        assert_eq!(WalletId::View(Chain::Ethereum, "0x789".to_string()).id(), "view_ethereum_0x789");
    }

    #[test]
    fn test_wallet_identifier_from_id() {
        assert!(matches!(WalletId::from_id("multicoin_0x123"), Some(WalletId::Multicoin(addr)) if addr == "0x123"));
        assert!(matches!(WalletId::from_id("single_ethereum_0x456"), Some(WalletId::Single(Chain::Ethereum, addr)) if addr == "0x456"));
        assert!(matches!(WalletId::from_id("privateKey_bitcoin_bc1"), Some(WalletId::PrivateKey(Chain::Bitcoin, addr)) if addr == "bc1"));
        assert!(matches!(WalletId::from_id("view_ethereum_0x789"), Some(WalletId::View(Chain::Ethereum, addr)) if addr == "0x789"));
        assert!(WalletId::from_id("invalid").is_none());
    }

    #[test]
    fn test_wallet_identifier_wallet_type() {
        assert_eq!(WalletId::Multicoin("0x123".to_string()).wallet_type(), WalletType::Multicoin);
        assert_eq!(WalletId::Single(Chain::Ethereum, "0x456".to_string()).wallet_type(), WalletType::Single);
        assert_eq!(WalletId::PrivateKey(Chain::Bitcoin, "bc1".to_string()).wallet_type(), WalletType::PrivateKey);
        assert_eq!(WalletId::View(Chain::Ethereum, "0x789".to_string()).wallet_type(), WalletType::View);
    }

    #[test]
    fn test_wallet_identifier_chain() {
        assert_eq!(WalletId::Multicoin("0x123".to_string()).chain(), None);
        assert_eq!(WalletId::Single(Chain::Ethereum, "0x456".to_string()).chain(), Some(Chain::Ethereum));
        assert_eq!(WalletId::PrivateKey(Chain::Bitcoin, "bc1".to_string()).chain(), Some(Chain::Bitcoin));
        assert_eq!(WalletId::View(Chain::Solana, "sol123".to_string()).chain(), Some(Chain::Solana));
    }

    #[test]
    fn test_wallet_identifier_serde() {
        let wallet_id = WalletId::Multicoin("0x8f348F300873Fd5DA36950B2aC75a26584584feE".to_string());
        let json = serde_json::to_string(&wallet_id).unwrap();
        assert_eq!(json, "\"multicoin_0x8f348F300873Fd5DA36950B2aC75a26584584feE\"");

        let parsed: WalletId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id(), wallet_id.id());
    }
}
