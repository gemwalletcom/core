use std::{collections::HashSet, error::Error};

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{asset_id::AssetId, asset_type::AssetType, AssetBasic, AssetProperties, AssetScore, Chain};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: AssetId,
    #[typeshare(skip)]
    pub chain: Chain,
    #[typeshare(skip)]
    pub token_id: Option<String>,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    #[serde(rename = "type")]
    pub asset_type: AssetType,
}

impl Chain {
    pub fn new_asset(&self, name: String, symbol: String, decimals: i32, asset_type: AssetType) -> Asset {
        Asset {
            id: self.as_asset_id(),
            chain: *self,
            token_id: None,
            name,
            symbol,
            decimals,
            asset_type,
        }
    }
}

impl Asset {
    pub fn new(id: AssetId, name: String, symbol: String, decimals: i32, asset_type: AssetType) -> Asset {
        Asset {
            id: id.clone(),
            chain: id.chain,
            token_id: id.token_id.clone(),
            name,
            symbol,
            decimals,
            asset_type,
        }
    }

    pub fn chain(&self) -> Chain {
        self.id.chain
    }

    pub fn full_name(&self) -> String {
        format!("{} ({})", self.name, self.symbol)
    }

    pub fn as_basic_primitive(&self) -> AssetBasic {
        AssetBasic::new(self.clone(), AssetProperties::default(self.id.clone()), AssetScore::default())
    }

    pub fn from_chain(chain: Chain) -> Asset {
        match chain {
            Chain::Ethereum => chain.new_asset("Ethereum".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Bitcoin => chain.new_asset("Bitcoin".to_string(), "BTC".to_string(), 8, AssetType::NATIVE),
            Chain::BitcoinCash => chain.new_asset("Bitcoin Cash".to_string(), "BCH".to_string(), 8, AssetType::NATIVE),
            Chain::Litecoin => chain.new_asset("Litecoin".to_string(), "LTC".to_string(), 8, AssetType::NATIVE),
            Chain::SmartChain => chain.new_asset("BNB Chain".to_string(), "BNB".to_string(), 18, AssetType::NATIVE),
            Chain::Polygon => chain.new_asset("Polygon".to_string(), "POL".to_string(), 18, AssetType::NATIVE),
            Chain::AvalancheC => chain.new_asset("Avalanche".to_string(), "AVAX".to_string(), 18, AssetType::NATIVE),
            Chain::Solana => chain.new_asset("Solana".to_string(), "SOL".to_string(), 9, AssetType::NATIVE),
            Chain::Thorchain => chain.new_asset("Thorchain".to_string(), "RUNE".to_string(), 8, AssetType::NATIVE),
            Chain::Cosmos => chain.new_asset("Cosmos".to_string(), "ATOM".to_string(), 6, AssetType::NATIVE),
            Chain::Osmosis => chain.new_asset("Osmosis".to_string(), "OSMO".to_string(), 6, AssetType::NATIVE),
            Chain::Celestia => chain.new_asset("Celestia".to_string(), "TIA".to_string(), 6, AssetType::NATIVE),
            Chain::Arbitrum => chain.new_asset("Arbitrum".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Ton => chain.new_asset("TON".to_string(), "TON".to_string(), 9, AssetType::NATIVE),
            Chain::Tron => chain.new_asset("TRON".to_string(), "TRX".to_string(), 6, AssetType::NATIVE),
            Chain::Doge => chain.new_asset("Dogecoin".to_string(), "DOGE".to_string(), 8, AssetType::NATIVE),
            Chain::Optimism => chain.new_asset("Optimism".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Aptos => chain.new_asset("Aptos".to_string(), "APT".to_string(), 8, AssetType::NATIVE),
            Chain::Base => chain.new_asset("Base".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Sui => chain.new_asset("Sui".to_string(), "SUI".to_string(), 9, AssetType::NATIVE),
            Chain::Xrp => chain.new_asset("XRP".to_string(), "XRP".to_string(), 6, AssetType::NATIVE),
            Chain::OpBNB => chain.new_asset("opBNB".to_string(), "BNB".to_string(), 18, AssetType::NATIVE),
            Chain::Fantom => chain.new_asset("Fantom".to_string(), "FTM".to_string(), 18, AssetType::NATIVE),
            Chain::Gnosis => chain.new_asset("Gnosis Chain".to_string(), "xDai".to_string(), 18, AssetType::NATIVE),
            Chain::Injective => chain.new_asset("Injective".to_string(), "INJ".to_string(), 18, AssetType::NATIVE),
            Chain::Sei => chain.new_asset("Sei".to_string(), "SEI".to_string(), 6, AssetType::NATIVE),
            Chain::Manta => chain.new_asset("Manta".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Blast => chain.new_asset("Blast".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Noble => chain.new_asset("Noble".to_string(), "USDC".to_string(), 6, AssetType::NATIVE),
            Chain::ZkSync => chain.new_asset("zkSync".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Linea => chain.new_asset("Linea".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Mantle => chain.new_asset("Mantle".to_string(), "MNT".to_string(), 18, AssetType::NATIVE),
            Chain::Celo => chain.new_asset("Celo".to_string(), "CELO".to_string(), 18, AssetType::NATIVE),
            Chain::Near => chain.new_asset("Near".to_string(), "NEAR".to_string(), 24, AssetType::NATIVE),
            Chain::World => chain.new_asset("World".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Stellar => chain.new_asset("Stellar".to_string(), "XLM".to_string(), 7, AssetType::NATIVE),
            Chain::Sonic => chain.new_asset("Sonic".to_string(), "S".to_string(), 18, AssetType::NATIVE),
            Chain::Algorand => chain.new_asset("Algorand".to_string(), "ALGO".to_string(), 6, AssetType::NATIVE),
            Chain::Polkadot => chain.new_asset("Polkadot".to_string(), "DOT".to_string(), 10, AssetType::NATIVE),
            Chain::Cardano => chain.new_asset("Cardano".to_string(), "ADA".to_string(), 6, AssetType::NATIVE),
            Chain::Abstract => chain.new_asset("Abstract".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Berachain => chain.new_asset("Berachain".to_string(), "BERA".to_string(), 18, AssetType::NATIVE),
            Chain::Ink => chain.new_asset("Ink".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Unichain => chain.new_asset("Unichain".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::Hyperliquid => chain.new_asset("HyperEVM".to_string(), "HYPE".to_string(), 18, AssetType::NATIVE),
            Chain::Monad => chain.new_asset("Monad Testnet".to_string(), "MON".to_string(), 18, AssetType::NATIVE),
        }
    }
}

pub trait AssetVecExt {
    fn ids(&self) -> Vec<AssetId>;
    fn ids_set(&self) -> HashSet<AssetId>;
    fn asset(&self, asset_id: AssetId) -> Option<Asset>;
    fn asset_result(&self, asset_id: AssetId) -> Result<&Asset, Box<dyn Error + Send + Sync>>;
}

impl AssetVecExt for Vec<Asset> {
    fn ids(&self) -> Vec<AssetId> {
        self.iter().map(|x| x.id.clone()).collect()
    }

    fn ids_set(&self) -> HashSet<AssetId> {
        self.iter().map(|x| x.id.clone()).collect()
    }

    fn asset(&self, asset_id: AssetId) -> Option<Asset> {
        self.iter().find(|x| x.id == asset_id).cloned()
    }

    fn asset_result(&self, asset_id: AssetId) -> Result<&Asset, Box<dyn Error + Send + Sync>> {
        self.iter().find(|x| x.id == asset_id).ok_or("Asset not found".into())
    }
}

pub trait AssetHashSetExt {
    fn ids(&self) -> Vec<String>;
}

impl AssetHashSetExt for HashSet<AssetId> {
    fn ids(&self) -> Vec<String> {
        self.iter().cloned().map(|x| x.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_id() {
        let asset = Asset::from_chain(Chain::Gnosis);

        assert_eq!(asset.symbol, "xDai");
    }
}
