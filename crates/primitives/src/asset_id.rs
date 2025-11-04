use std::{collections::HashSet, fmt};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::{AssetSubtype, chain::Chain};

pub const TOKEN_ID_SEPARATOR: &str = "::";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetId {
    pub chain: Chain,
    pub token_id: Option<String>,
}

impl Serialize for AssetId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for AssetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        AssetId::new(&s).ok_or_else(|| de::Error::custom("Invalid AssetId"))
    }
}

impl From<&str> for AssetId {
    fn from(value: &str) -> Self {
        AssetId::new(value).unwrap()
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match &self.token_id {
            Some(token_id) => {
                format!("{}_{}", self.chain.as_ref(), token_id)
            }
            None => self.chain.as_ref().to_owned(),
        };
        write!(f, "{str}")
    }
}

impl From<AssetId> for String {
    fn from(value: AssetId) -> Self {
        value.to_string()
    }
}

impl AssetId {
    pub fn new(asset_id: &str) -> Option<Self> {
        let split: Vec<&str> = asset_id.split('_').collect();
        if split.len() == 1 {
            if let Ok(chain) = asset_id.parse::<Chain>() {
                return Some(AssetId { chain, token_id: None });
            }
        } else if split.len() >= 2
            && let Ok(chain) = split[0].parse::<Chain>()
        {
            return Some(AssetId {
                chain,
                token_id: Some(split[1..split.len()].join("_")),
            });
        }
        None
    }

    pub fn from(chain: Chain, token_id: Option<String>) -> AssetId {
        AssetId { chain, token_id }
    }

    pub fn from_token(chain: Chain, token_id: &str) -> AssetId {
        AssetId {
            chain,
            token_id: Some(token_id.to_string()),
        }
    }

    pub fn from_chain(chain: Chain) -> AssetId {
        AssetId { chain, token_id: None }
    }

    pub fn sub_token_id(ids: &[String]) -> String {
        ids.join(TOKEN_ID_SEPARATOR)
    }

    pub fn decode_token_id(token_id: &str) -> Vec<String> {
        token_id.split(TOKEN_ID_SEPARATOR).map(|s| s.to_string()).collect()
    }

    pub fn split_token_id(token_id: &str, separator: char) -> Vec<String> {
        token_id.split(separator).map(|s| s.to_string()).collect()
    }

    pub fn get_token_id(&self) -> Result<&String, crate::SignerError> {
        self.token_id
            .as_ref()
            .ok_or_else(|| crate::SignerError::InvalidInput("Token ID required".to_string()))
    }

    pub fn split_token_parts(&self, separator: char) -> Result<(String, String), crate::SignerError> {
        let token_id = self.get_token_id()?;
        let parts: Vec<&str> = token_id.split(separator).collect();
        if parts.len() < 2 {
            return Err(crate::SignerError::InvalidInput(format!("Invalid token ID format: {}", token_id)));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    pub fn is_native(&self) -> bool {
        self.token_id.is_none()
    }
    pub fn is_token(&self) -> bool {
        self.token_id.is_some()
    }

    pub fn token_subtype(&self) -> AssetSubtype {
        if self.is_native() { AssetSubtype::NATIVE } else { AssetSubtype::TOKEN }
    }

    pub fn token_components(&self) -> Option<(String, Option<String>, Option<i32>)> {
        let token_id = self.token_id.as_ref()?;
        let parts = AssetId::decode_token_id(token_id);
        if parts.is_empty() {
            return None;
        }

        let symbol = parts[0].clone();
        let contract = parts.get(1).and_then(|value| (!value.is_empty()).then(|| value.clone()));
        let index = parts.get(2).and_then(|value| value.parse::<i32>().ok());

        Some((symbol, contract, index))
    }
}

pub trait AssetIdVecExt {
    fn ids(&self) -> Vec<String>;
    fn ids_set(&self) -> HashSet<AssetId>;
}

impl AssetIdVecExt for Vec<AssetId> {
    fn ids(&self) -> Vec<String> {
        self.iter().map(|x| x.to_string()).collect()
    }

    fn ids_set(&self) -> HashSet<AssetId> {
        self.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_asset_id_with_coin() {
        let asset_id = AssetId::new("ethereum").unwrap();
        assert_eq!(asset_id.chain, Chain::Ethereum);
        assert_eq!(asset_id.token_id, None);
    }

    #[test]
    fn test_new_asset_id_with_coin_and_token() {
        let asset_id = AssetId::new("ethereum_0x1234567890abcdef").unwrap();
        assert_eq!(asset_id.chain, Chain::Ethereum);
        assert_eq!(asset_id.token_id, Some("0x1234567890abcdef".to_owned()));
    }

    #[test]
    fn test_new_asset_id_with_coin_and_token_extra_underscore() {
        let asset_id = AssetId::new("ton_EQAvlWFDxGF2lXm67y4yzC17wYKD9A0guwPkMs1gOsM__NOT").unwrap();
        assert_eq!(asset_id.chain, Chain::Ton);
        assert_eq!(asset_id.token_id, Some("EQAvlWFDxGF2lXm67y4yzC17wYKD9A0guwPkMs1gOsM__NOT".to_owned()));
    }

    #[test]
    fn test_sub_token_id() {
        let result = AssetId::sub_token_id(&["test".to_string()]);
        assert_eq!(result, "test");

        let result = AssetId::sub_token_id(&["perpetual".to_string(), "BTC".to_string()]);
        assert_eq!(result, "perpetual::BTC");

        let result = AssetId::sub_token_id(&["type".to_string(), "subtype".to_string(), "coin".to_string()]);
        assert_eq!(result, "type::subtype::coin");

        let result = AssetId::sub_token_id(&[]);
        assert_eq!(result, "");

        let asset_id = AssetId::from(Chain::HyperCore, Some(AssetId::sub_token_id(&["perpetual".to_string(), "ETH".to_string()])));
        assert_eq!(asset_id.chain, Chain::HyperCore);
        assert_eq!(asset_id.token_id, Some("perpetual::ETH".to_string()));
        assert_eq!(asset_id.to_string(), "hypercore_perpetual::ETH");
    }

    #[test]
    fn test_decode_token_id() {
        assert_eq!(AssetId::decode_token_id("USDC"), vec!["USDC"]);
        assert_eq!(
            AssetId::decode_token_id("USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0"),
            vec!["USDC", "0x6d1e7cde53ba9467b783cb7c530ce054", "0"]
        );
        assert_eq!(AssetId::decode_token_id("perpetual::BTC"), vec!["perpetual", "BTC"]);
        assert_eq!(AssetId::decode_token_id(""), vec![""]);
    }
}
