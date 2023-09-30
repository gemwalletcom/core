use typeshare::typeshare;
use serde::{Serialize, Deserialize};

use crate::chain::Chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
pub struct AssetId {
    pub chain: Chain,
    #[serde(rename = "tokenId")]
    pub token_id: Option<String>,
}

impl AssetId {
    pub fn new(asset_id: &str) -> Option<Self> {
        let parts: Vec<&str> = asset_id.split('_').collect();
        if parts.is_empty() || parts.len() > 2 {
            return None;
        }
        let chain = Chain::from_str(parts[0])?;
        let token_id = parts.get(1).map(|s| s.to_owned());
        Some(Self { chain, token_id: token_id.map(|s| s.to_owned()) })
    }

    pub fn to_string(&self) -> String {
        match &self.token_id {
            Some(token_id) => {
                format!("{}_{}", self.chain.as_str(), token_id)
            }
            None => self.chain.as_str().to_owned(),
        }
    }

    pub fn from_chain(chain: Chain) -> AssetId {
        AssetId { chain, token_id: None }
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
}