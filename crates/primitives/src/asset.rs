use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{asset_id::AssetId, asset_type::AssetType, Chain};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct Asset {
    pub id: AssetId,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    #[serde(rename = "type")]
    pub asset_type: AssetType,
}

impl Asset {
    pub fn chain(&self) -> Chain {
        self.id.chain
    }

    pub fn full_name(&self) -> String {
        format!("{} ({})", self.name, self.symbol)
    }

    pub fn from_chain(chain: Chain) -> Asset {
        match chain {
            Chain::Ethereum => Asset {
                id: chain.as_asset_id(),
                name: "Ethereum".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Bitcoin => Asset {
                id: chain.as_asset_id(),
                name: "Bitcoin".to_string(),
                symbol: "BTC".to_string(),
                decimals: 8,
                asset_type: AssetType::NATIVE,
            },
            Chain::BitcoinCash => Asset {
                id: chain.as_asset_id(),
                name: "Bitcoin Cash".to_string(),
                symbol: "BCH".to_string(),
                decimals: 8,
                asset_type: AssetType::NATIVE,
            },
            Chain::Litecoin => Asset {
                id: chain.as_asset_id(),
                name: "Litecoin".to_string(),
                symbol: "LTC".to_string(),
                decimals: 8,
                asset_type: AssetType::NATIVE,
            },
            Chain::SmartChain => Asset {
                id: chain.as_asset_id(),
                name: "Smart Chain".to_string(),
                symbol: "BNB".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Polygon => Asset {
                id: chain.as_asset_id(),
                name: "Polygon".to_string(),
                symbol: "POL".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::AvalancheC => Asset {
                id: chain.as_asset_id(),
                name: "Avalanche".to_string(),
                symbol: "AVAX".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Solana => Asset {
                id: chain.as_asset_id(),
                name: "Solana".to_string(),
                symbol: "SOL".to_string(),
                decimals: 9,
                asset_type: AssetType::NATIVE,
            },
            Chain::Thorchain => Asset {
                id: chain.as_asset_id(),
                name: "Thorchain".to_string(),
                symbol: "RUNE".to_string(),
                decimals: 8,
                asset_type: AssetType::NATIVE,
            },
            Chain::Cosmos => Asset {
                id: chain.as_asset_id(),
                name: "Cosmos".to_string(),
                symbol: "ATOM".to_string(),
                decimals: 6,
                asset_type: AssetType::NATIVE,
            },
            Chain::Osmosis => Asset {
                id: chain.as_asset_id(),
                name: "Osmosis".to_string(),
                symbol: "OSMO".to_string(),
                decimals: 6,
                asset_type: AssetType::NATIVE,
            },
            Chain::Celestia => Asset {
                id: chain.as_asset_id(),
                name: "Celestia".to_string(),
                symbol: "TIA".to_string(),
                decimals: 6,
                asset_type: AssetType::NATIVE,
            },
            Chain::Arbitrum => Asset {
                id: chain.as_asset_id(),
                name: "Arbitrum".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Ton => Asset {
                id: chain.as_asset_id(),
                name: "TON".to_string(),
                symbol: "TON".to_string(),
                decimals: 9,
                asset_type: AssetType::NATIVE,
            },
            Chain::Tron => Asset {
                id: chain.as_asset_id(),
                name: "TRON".to_string(),
                symbol: "TRX".to_string(),
                decimals: 6,
                asset_type: AssetType::NATIVE,
            },
            Chain::Doge => Asset {
                id: chain.as_asset_id(),
                name: "Dogecoin".to_string(),
                symbol: "DOGE".to_string(),
                decimals: 8,
                asset_type: AssetType::NATIVE,
            },
            Chain::Optimism => Asset {
                id: chain.as_asset_id(),
                name: "Optimism".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Aptos => Asset {
                id: chain.as_asset_id(),
                name: "Aptos".to_string(),
                symbol: "APT".to_string(),
                decimals: 8,
                asset_type: AssetType::NATIVE,
            },
            Chain::Base => Asset {
                id: chain.as_asset_id(),
                name: "Base".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Sui => Asset {
                id: chain.as_asset_id(),
                name: "Sui".to_string(),
                symbol: "SUI".to_string(),
                decimals: 9,
                asset_type: AssetType::NATIVE,
            },
            Chain::Xrp => Asset {
                id: chain.as_asset_id(),
                name: "XRP".to_string(),
                symbol: "XRP".to_string(),
                decimals: 6,
                asset_type: AssetType::NATIVE,
            },
            Chain::OpBNB => Asset {
                id: chain.as_asset_id(),
                name: "opBNB".to_string(),
                symbol: "BNB".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Fantom => Asset {
                id: chain.as_asset_id(),
                name: "Fantom".to_string(),
                symbol: "FTM".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Gnosis => Asset {
                id: chain.as_asset_id(),
                name: "Gnosis Chain".to_string(),
                symbol: "xDai".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Injective => Asset {
                id: chain.as_asset_id(),
                name: "Injective".to_string(),
                symbol: "INJ".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Sei => Asset {
                id: chain.as_asset_id(),
                name: "Sei".to_string(),
                symbol: "SEI".to_string(),
                decimals: 6,
                asset_type: AssetType::NATIVE,
            },
            Chain::Manta => Asset {
                id: chain.as_asset_id(),
                name: "Manta".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Blast => Asset {
                id: chain.as_asset_id(),
                name: "Blast".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Noble => Asset {
                id: chain.as_asset_id(),
                name: "Noble".to_string(),
                symbol: "USDC".to_string(),
                decimals: 6,
                asset_type: AssetType::NATIVE,
            },
            Chain::ZkSync => Asset {
                id: chain.as_asset_id(),
                name: "zkSync".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Linea => Asset {
                id: chain.as_asset_id(),
                name: "Linea".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Mantle => Asset {
                id: chain.as_asset_id(),
                name: "Mantle".to_string(),
                symbol: "MNT".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Celo => Asset {
                id: chain.as_asset_id(),
                name: "Celo".to_string(),
                symbol: "CELO".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Near => Asset {
                id: chain.as_asset_id(),
                name: "Near".to_string(),
                symbol: "NEAR".to_string(),
                decimals: 24,
                asset_type: AssetType::NATIVE,
            },
            Chain::World => Asset {
                id: chain.as_asset_id(),
                name: "World".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Stellar => Asset {
                id: chain.as_asset_id(),
                name: "Stellar".to_string(),
                symbol: "XLM".to_string(),
                decimals: 7,
                asset_type: AssetType::NATIVE,
            },
            Chain::Sonic => Asset {
                id: chain.as_asset_id(),
                name: "Sonic".to_string(),
                symbol: "S".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Algorand => Asset {
                id: chain.as_asset_id(),
                name: "Algorand".to_string(),
                symbol: "ALGO".to_string(),
                decimals: 6,
                asset_type: AssetType::NATIVE,
            },
            Chain::Polkadot => Asset {
                id: chain.as_asset_id(),
                name: "Polkadot".to_string(),
                symbol: "DOT".to_string(),
                decimals: 10,
                asset_type: AssetType::NATIVE,
            },
            Chain::Cardano => Asset {
                id: chain.as_asset_id(),
                name: "Cardano".to_string(),
                symbol: "ADA".to_string(),
                decimals: 6,
                asset_type: AssetType::NATIVE,
            },
            Chain::Abstract => Asset {
                id: chain.as_asset_id(),
                name: "Abstract".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Berachain => Asset {
                id: chain.as_asset_id(),
                name: "Berachain".to_string(),
                symbol: "BERA".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Ink => Asset {
                id: chain.as_asset_id(),
                name: "Ink".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Unichain => Asset {
                id: chain.as_asset_id(),
                name: "Unichain".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Hyperliquid => Asset {
                id: chain.as_asset_id(),
                name: "HypeEVM".to_string(),
                symbol: "HYPE".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
            Chain::Monad => Asset {
                id: chain.as_asset_id(),
                name: "Monad Testnet".to_string(), //TODO: Monad
                symbol: "MON".to_string(),
                decimals: 18,
                asset_type: AssetType::NATIVE,
            },
        }
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
