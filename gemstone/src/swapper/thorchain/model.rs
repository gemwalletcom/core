use primitives::Chain;
use serde::{Deserialize, Serialize};

pub enum ThorChainAsset {
    Doge,
    Thorchain,
    Ethereum,
    Cosmos,
    Bitcoin,
    Litecoin,
    SmartChain,
    AvalancheC,
}

// https://dev.thorchain.org/concepts/memo-length-reduction.html
impl ThorChainAsset {
    pub fn short_name(&self) -> &str {
        match self {
            ThorChainAsset::Doge => "d",               // DOGE.DOGE
            ThorChainAsset::Thorchain => "r",          // THOR.RUNE
            ThorChainAsset::Ethereum => "e",           // "ETH.ETH"
            ThorChainAsset::Cosmos => "g",             // GAIA.ATOM
            ThorChainAsset::Bitcoin => "b",            // BTC.BTC
            ThorChainAsset::Litecoin => "l",           // LTC.LTC
            ThorChainAsset::SmartChain => "s",         // BSC.BNB
            ThorChainAsset::AvalancheC => "AVAX.AVAX", // AVAX.AVAX. not sure
        }
    }

    pub fn chain(&self) -> Chain {
        match self {
            ThorChainAsset::Doge => Chain::Doge,
            ThorChainAsset::Thorchain => Chain::Thorchain,
            ThorChainAsset::Ethereum => Chain::Ethereum,
            ThorChainAsset::Cosmos => Chain::Cosmos,
            ThorChainAsset::Bitcoin => Chain::Bitcoin,
            ThorChainAsset::Litecoin => Chain::Litecoin,
            ThorChainAsset::SmartChain => Chain::SmartChain,
            ThorChainAsset::AvalancheC => Chain::AvalancheC,
        }
    }

    pub fn from_chain(chain: &Chain) -> Option<ThorChainAsset> {
        match chain {
            Chain::Thorchain => Some(ThorChainAsset::Thorchain),
            Chain::Doge => Some(ThorChainAsset::Doge),
            Chain::Cosmos => Some(ThorChainAsset::Cosmos),
            Chain::Bitcoin => Some(ThorChainAsset::Bitcoin),
            Chain::Litecoin => Some(ThorChainAsset::Litecoin),
            Chain::SmartChain => Some(ThorChainAsset::SmartChain),
            Chain::Ethereum => Some(ThorChainAsset::Ethereum),
            Chain::AvalancheC => Some(ThorChainAsset::AvalancheC),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSwapRequest {
    pub from_asset: String,
    pub to_asset: String,
    pub amount: String,
    pub affiliate: String,
    pub affiliate_bps: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSwapResponse {
    pub expected_amount_out: String,
    pub inbound_address: Option<String>,
    pub fees: QuoteFees,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteFees {}
