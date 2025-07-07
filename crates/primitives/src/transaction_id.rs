use crate::chain::Chain;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionId {
    pub chain: Chain,
    pub hash: String,
}

impl TransactionId {
    pub fn new(chain: Chain, hash: String) -> Self {
        Self { chain, hash }
    }
}

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.chain.as_ref(), self.hash)
    }
}

impl Serialize for TransactionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TransactionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TransactionId::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for TransactionId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid TransactionId format: expected chain_hash, got {s}"));
        }

        let chain_str = parts[0];
        let hash_str = parts[1];

        let chain = Chain::from_str(chain_str).map_err(|e| format!("Invalid chain identifier '{chain_str}': {e}"))?;

        Ok(TransactionId::new(chain, hash_str.to_string()))
    }
}

impl TryFrom<String> for TransactionId {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        TransactionId::from_str(&s)
    }
}

impl TryFrom<&str> for TransactionId {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        TransactionId::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chain;
    use serde_json;
    use std::convert::TryFrom;

    #[test]
    fn test_display_trait_to_string() {
        let tx_id = TransactionId::new(Chain::Ethereum, "0x123".to_string());
        assert_eq!(tx_id.to_string(), "ethereum_0x123"); // This now uses Display::to_string()
        assert_eq!(format!("{tx_id}"), "ethereum_0x123"); // Also test format!()
    }

    #[test]
    fn test_serde_roundtrip() {
        let tx_id = TransactionId::new(Chain::Solana, "solhash456".to_string());
        let serialized = serde_json::to_string(&tx_id).unwrap();
        assert_eq!(serialized, "\"solana_solhash456\"");
        let deserialized: TransactionId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx_id, deserialized);
    }

    #[test]
    fn test_from_str_valid() {
        let tx_id_str = "bitcoin_btchash789";
        let tx_id = TransactionId::from_str(tx_id_str).unwrap();
        assert_eq!(tx_id.chain, Chain::Bitcoin);
        assert_eq!(tx_id.hash, "btchash789");
    }

    #[test]
    fn test_from_str_invalid_format() {
        let result = TransactionId::from_str("invalidformat");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_invalid_chain() {
        let result = TransactionId::from_str("nonexistentchain_somehash");
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_string_valid() {
        let tx_id_str = "ethereum_0xabc".to_string();
        let tx_id = TransactionId::try_from(tx_id_str).unwrap();
        assert_eq!(tx_id.chain, Chain::Ethereum);
        assert_eq!(tx_id.hash, "0xabc");
    }

    #[test]
    fn test_try_from_str_ref_valid() {
        let tx_id_str = "polygon_0xdef";
        let tx_id = TransactionId::try_from(tx_id_str).unwrap();
        assert_eq!(tx_id.chain, Chain::Polygon);
        assert_eq!(tx_id.hash, "0xdef");
    }
}
