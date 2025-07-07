use bigdecimal::BigDecimal;
use primitives::{Chain, DeFiPositionType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// DeBank API specific models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankChain {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankTokenItem {
    pub id: String,
    pub chain: String,
    pub name: String,
    pub symbol: String,
    pub display_symbol: Option<String>,
    pub optimized_symbol: Option<String>,
    pub decimals: u8,
    pub logo_url: Option<String>,
    pub protocol_id: Option<String>,
    pub price: Option<BigDecimal>,
    pub amount: BigDecimal,
    pub is_verified: Option<bool>,
    pub is_core: Option<bool>,
    pub is_wallet: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankProtocol {
    pub id: String,
    pub chain: String,
    pub name: String,
    pub site_url: Option<String>,
    pub logo_url: Option<String>,
    pub has_supported_portfolio: bool,
    pub tvl: Option<BigDecimal>,
    pub portfolio_item_list: Option<Vec<DeBankPortfolioItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankPortfolioItem {
    pub stats: DeBankPortfolioStats,
    pub asset_dict: Option<HashMap<String, BigDecimal>>,
    pub asset_token_list: Option<Vec<DeBankTokenItem>>,
    pub name: String,
    pub detail_types: Vec<String>,
    pub detail: DeBankPortfolioDetail,
    pub proxy_detail: Option<serde_json::Value>,
    pub pool: Option<DeBankPool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankPortfolioStats {
    pub asset_usd_value: BigDecimal,
    pub debt_usd_value: BigDecimal,
    pub net_usd_value: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankPortfolioDetail {
    pub supply_token_list: Option<Vec<DeBankTokenItem>>,
    pub reward_token_list: Option<Vec<DeBankTokenItem>>,
    pub borrow_token_list: Option<Vec<DeBankTokenItem>>,
    pub health_rate: Option<BigDecimal>,
    pub description: Option<String>,
    pub unlock_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankPool {
    pub id: String,
    pub chain: String,
    pub project_id: String,
    pub adapter_id: String,
    pub controller: String,
    pub index: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

// Internal mapping structures
pub struct DeBankMapping;

impl DeBankMapping {
    pub fn chain_to_debank_id(chain: &Chain) -> Option<&'static str> {
        match chain {
            Chain::Ethereum => Some("eth"),
            Chain::Polygon => Some("matic"),
            Chain::SmartChain => Some("bsc"),
            Chain::AvalancheC => Some("avax"),
            Chain::Arbitrum => Some("arb"),
            Chain::Optimism => Some("op"),
            Chain::Fantom => Some("ftm"),
            Chain::Base => Some("base"),
            Chain::Linea => Some("linea"),
            Chain::ZkSync => Some("era"),
            Chain::Manta => Some("manta"),
            Chain::Celo => Some("celo"),
            Chain::Mantle => Some("mnt"),
            Chain::Sonic => Some("sonic"),
            Chain::Gnosis => Some("xdai"),
            Chain::Blast => Some("blast"),
            _ => None,
        }
    }

    pub fn debank_id_to_chain(chain_id: &str) -> Option<Chain> {
        match chain_id {
            "eth" => Some(Chain::Ethereum),
            "bsc" => Some(Chain::SmartChain),
            "base" => Some(Chain::Base),
            "arb" => Some(Chain::Arbitrum),
            "matic" => Some(Chain::Polygon),
            "avax" => Some(Chain::AvalancheC),
            "op" => Some(Chain::Optimism),
            "mnt" => Some(Chain::Mantle),
            "ftm" => Some(Chain::Fantom),
            "sonic" => Some(Chain::Sonic),
            "xdai" => Some(Chain::Gnosis),
            "blast" => Some(Chain::Blast),
            "linea" => Some(Chain::Linea),
            "era" => Some(Chain::ZkSync),
            "manta" => Some(Chain::Manta),
            "celo" => Some(Chain::Celo),
            _ => None,
        }
    }

    pub fn position_type_from_detail_type(detail_type: &str) -> DeFiPositionType {
        match detail_type {
            "common" => DeFiPositionType::Wallet,
            "lending" => DeFiPositionType::Lending,
            "liquidity_pool" => DeFiPositionType::Liquidity,
            "farming" => DeFiPositionType::Farming,
            "staked" => DeFiPositionType::Staking,
            "locked" => DeFiPositionType::Locked,
            "vesting" => DeFiPositionType::Vesting,
            "perpetuals" => DeFiPositionType::Perpetual,
            "options" => DeFiPositionType::Options,
            "leveraged_farming" => DeFiPositionType::Leverage,
            "insurance" => DeFiPositionType::Vault,
            _ => DeFiPositionType::Wallet,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_protocol_list() {
        let json = include_str!("./protocol_list.json");
        let protocol_list: Vec<DeBankProtocol> = serde_json::from_str(json).unwrap();

        println!("{protocol_list:?}");
    }
}
