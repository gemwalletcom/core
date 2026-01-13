use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typeshare::typeshare;

use crate::chain::Chain;
use crate::chain_address::ChainAddress;
use crate::device::Device;
use crate::wallet::WalletSource;
use crate::wallet_type::WalletType;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct Subscription {
    pub wallet_index: i32,
    pub chain: Chain,
    pub address: String,
}

#[derive(Clone, Debug)]
pub enum WalletIdType {
    Multicoin(String),
    Single(Chain, String),
    PrivateKey(Chain, String),
    View(Chain, String),
}

impl Serialize for WalletIdType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.id())
    }
}

impl<'de> Deserialize<'de> for WalletIdType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        WalletIdType::from_id(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid wallet id: {}", s)))
    }
}

impl WalletIdType {
    pub fn id(&self) -> String {
        match self {
            WalletIdType::Multicoin(address) => format!("{}_{}", WalletType::Multicoin.as_ref(), address),
            WalletIdType::Single(chain, address)
            | WalletIdType::PrivateKey(chain, address)
            | WalletIdType::View(chain, address) => format!("{}_{}_{}", self.wallet_type().as_ref(), chain.as_ref(), address),
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        let parts: Vec<&str> = id.splitn(2, '_').collect();
        if parts.len() != 2 {
            return None;
        }
        let wallet_type: WalletType = parts[0].parse().ok()?;
        let rest = parts[1];

        match wallet_type {
            WalletType::Multicoin => Some(WalletIdType::Multicoin(rest.to_string())),
            WalletType::Single | WalletType::PrivateKey | WalletType::View => {
                let chain_parts: Vec<&str> = rest.splitn(2, '_').collect();
                if chain_parts.len() != 2 {
                    return None;
                }
                let chain: Chain = chain_parts[0].parse().ok()?;
                let address = chain_parts[1].to_string();
                match wallet_type {
                    WalletType::Single => Some(WalletIdType::Single(chain, address)),
                    WalletType::PrivateKey => Some(WalletIdType::PrivateKey(chain, address)),
                    WalletType::View => Some(WalletIdType::View(chain, address)),
                    _ => None,
                }
            }
        }
    }

    pub fn wallet_type(&self) -> WalletType {
        match self {
            WalletIdType::Multicoin(_) => WalletType::Multicoin,
            WalletIdType::Single(_, _) => WalletType::Single,
            WalletIdType::PrivateKey(_, _) => WalletType::PrivateKey,
            WalletIdType::View(_, _) => WalletType::View,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct WalletSubscription {
    #[typeshare(serialized_as = "String")]
    pub wallet_id: WalletIdType,
    pub source: WalletSource,
    pub subscriptions: Vec<ChainAddress>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceSubscription {
    pub device: Device,
    pub subscription: Subscription,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_id_type_id() {
        assert_eq!(WalletIdType::Multicoin("0x123".to_string()).id(), "multicoin_0x123");
        assert_eq!(WalletIdType::Single(Chain::Ethereum, "0x456".to_string()).id(), "single_ethereum_0x456");
        assert_eq!(WalletIdType::PrivateKey(Chain::Bitcoin, "bc1".to_string()).id(), "privateKey_bitcoin_bc1");
        assert_eq!(WalletIdType::View(Chain::Ethereum, "0x789".to_string()).id(), "view_ethereum_0x789");
    }

    #[test]
    fn test_wallet_id_type_from_id() {
        assert!(matches!(WalletIdType::from_id("multicoin_0x123"), Some(WalletIdType::Multicoin(addr)) if addr == "0x123"));
        assert!(matches!(WalletIdType::from_id("single_ethereum_0x456"), Some(WalletIdType::Single(Chain::Ethereum, addr)) if addr == "0x456"));
        assert!(matches!(WalletIdType::from_id("privateKey_bitcoin_bc1"), Some(WalletIdType::PrivateKey(Chain::Bitcoin, addr)) if addr == "bc1"));
        assert!(matches!(WalletIdType::from_id("view_ethereum_0x789"), Some(WalletIdType::View(Chain::Ethereum, addr)) if addr == "0x789"));
        assert!(WalletIdType::from_id("invalid").is_none());
    }

    #[test]
    fn test_wallet_id_type_wallet_type() {
        assert_eq!(WalletIdType::Multicoin("0x123".to_string()).wallet_type(), WalletType::Multicoin);
        assert_eq!(WalletIdType::Single(Chain::Ethereum, "0x456".to_string()).wallet_type(), WalletType::Single);
        assert_eq!(WalletIdType::PrivateKey(Chain::Bitcoin, "bc1".to_string()).wallet_type(), WalletType::PrivateKey);
        assert_eq!(WalletIdType::View(Chain::Ethereum, "0x789".to_string()).wallet_type(), WalletType::View);
    }

    #[test]
    fn test_wallet_id_type_serde() {
        let wallet_id = WalletIdType::Multicoin("0x8f348F300873Fd5DA36950B2aC75a26584584feE".to_string());
        let json = serde_json::to_string(&wallet_id).unwrap();
        assert_eq!(json, "\"multicoin_0x8f348F300873Fd5DA36950B2aC75a26584584feE\"");

        let parsed: WalletIdType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id(), wallet_id.id());
    }
}
