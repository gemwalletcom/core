use std::{collections::HashSet, fmt};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::chain::Chain;

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
        write!(f, "{}", str)
    }
}

impl AssetId {
    pub fn new(asset_id: &str) -> Option<Self> {
        let split: Vec<&str> = asset_id.split('_').collect();
        if split.len() == 1 {
            if let Ok(chain) = asset_id.parse::<Chain>() {
                return Some(AssetId { chain, token_id: None });
            }
        } else if split.len() >= 2 {
            if let Ok(chain) = split[0].parse::<Chain>() {
                return Some(AssetId {
                    chain,
                    token_id: Some(split[1..split.len()].join("_")),
                });
            }
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

    pub fn is_native(&self) -> bool {
        self.token_id.is_none()
    }
    pub fn is_token(&self) -> bool {
        self.token_id.is_some()
    }
}

pub trait AssetIdVecExt {
    fn ids(&self) -> Vec<String>;
}

impl AssetIdVecExt for Vec<AssetId> {
    fn ids(&self) -> Vec<String> {
        self.iter().map(|x| x.to_string()).collect()
    }
}

pub trait AssetIdHashSetExt {
    fn ids(&self) -> Vec<String>;
}

impl AssetIdHashSetExt for HashSet<AssetId> {
    fn ids(&self) -> Vec<String> {
        self.iter().map(|x| x.to_string()).collect()
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
}
