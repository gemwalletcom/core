use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::chain::Chain;
use gem_evm::address::EthereumAddress;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
pub struct AssetId {
    pub chain: Chain,
    #[serde(rename = "tokenId")]
    pub token_id: Option<String>,
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
        let parts: Vec<&str> = asset_id.split('_').collect();
        if parts.is_empty() || parts.len() > 2 {
            return None;
        }
        let chain = Chain::from_str(parts[0]).ok()?;
        let token_id = parts.get(1).map(|s| s.to_owned());
        Some(Self {
            chain,
            token_id: token_id.map(|s| s.to_owned()),
        })
    }

    pub fn from(chain: Chain, token_id: Option<String>) -> AssetId {
        AssetId { chain, token_id }
    }

    pub fn from_chain(chain: Chain) -> AssetId {
        AssetId {
            chain,
            token_id: None,
        }
    }

    pub fn is_native(&self) -> bool {
        self.token_id.is_none()
    }

    pub fn format_token_id(chain: Chain, token_id: String) -> Option<String> {
        match chain {
            Chain::Ethereum
            | Chain::SmartChain
            | Chain::Polygon
            | Chain::Arbitrum
            | Chain::Optimism
            | Chain::Base
            | Chain::AvalancheC
            | Chain::OpBNB
            | Chain::Fantom
            | Chain::Gnosis
            | Chain::Manta
            | Chain::Blast => Some(EthereumAddress::parse(&token_id)?.to_checksum()),
            Chain::Solana | Chain::Sui => Some(token_id),
            Chain::Tron => {
                if token_id.len() == 34 && token_id.starts_with('T') {
                    Some(token_id)
                } else {
                    None
                }
            }
            Chain::Bitcoin
            | Chain::Litecoin
            | Chain::Binance
            | Chain::Thorchain
            | Chain::Cosmos
            | Chain::Osmosis
            | Chain::Celestia
            | Chain::Ton
            | Chain::Doge
            | Chain::Aptos
            | Chain::Xrp
            | Chain::Injective
            | Chain::Noble
            | Chain::Sei => None,
        }
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
