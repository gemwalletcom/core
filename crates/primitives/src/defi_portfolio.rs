use crate::{AssetId, Chain};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::EnumString;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPortfolio {
    pub address: String,
    pub chain_id: String,
    pub summary: PortfolioSummary,
    pub positions: Vec<DeFiPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub total_value_usd: BigDecimal,
    pub value_by_protocol: HashMap<String, BigDecimal>,
    pub total_yield_usd: Option<BigDecimal>,
    pub health_score: Option<f64>,
    pub performance: Option<PortfolioPerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPerformance {
    pub change_24h: Option<BigDecimal>,
    pub change_7d: Option<BigDecimal>,
    pub change_30d: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPosition {
    pub id: String,
    pub address: String,
    pub chain_id: String,
    pub protocol: DeFiProtocol,
    pub position_type: DeFiPositionType,
    pub name: String,
    pub stats: PositionStats,
    pub assets: Vec<DeFiAsset>,
    pub metadata: PositionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiProtocol {
    pub id: String,
    pub name: String,
    pub category: DeFiProtocolCategory,
    pub logo_url: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DeFiPositionType {
    Wallet,
    Staking,
    Lending,
    Liquidity,
    Farming,
    Vault,
    Perpetual,
    Options,
    Leverage,
    Bridge,
    Governance,
    Vesting,
    Airdrop,
    Nft,
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DeFiProtocolCategory {
    Dex,
    Lending,
    Staking,
    Yield,
    Derivatives,
    Insurance,
    Bridge,
    Wallet,
    Nft,
    Governance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionStats {
    pub total_value_usd: BigDecimal,
    pub asset_value_usd: BigDecimal,
    pub debt_value_usd: BigDecimal,
    pub net_value_usd: BigDecimal,
    pub rewards_value_usd: Option<BigDecimal>,
    pub daily_yield_usd: Option<BigDecimal>,
    pub apy: Option<BigDecimal>,
    pub health_ratio: Option<BigDecimal>,
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiAsset {
    pub asset_id: AssetId,
    pub amount: BigDecimal,
    pub value_usd: BigDecimal,
    pub asset_type: DeFiAssetType,
    pub yield_info: Option<YieldInfo>,
    pub attributes: Option<AssetAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DeFiAssetType {
    Supply,
    Borrow,
    Reward,
    Liquidity,
    Staked,
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldInfo {
    pub apr: BigDecimal,
    pub apy: BigDecimal,
    pub source: Option<YieldSource>,
    pub rewards: Option<Vec<YieldReward>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum YieldSource {
    Staking,
    Lending,
    Farming,
    TradingFees,
    ProtocolEmissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldReward {
    pub asset_id: AssetId,
    pub amount: BigDecimal,
    pub value_usd: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAttributes {
    pub is_locked: Option<bool>,
    pub unlock_at: Option<DateTime<Utc>>,
    pub is_claimable: Option<bool>,
    pub is_deprecated: Option<bool>,
    pub ltv: Option<BigDecimal>,
    pub liquidation_threshold: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionMetadata {
    pub created_at: Option<DateTime<Utc>>,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub last_tx_hash: Option<String>,
    pub protocol_position_id: Option<String>,
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPositionFilters {
    pub position_types: Option<Vec<DeFiPositionType>>,
    pub chains: Vec<Chain>,
    pub protocols: Option<Vec<String>>,
    pub has_debt: Option<bool>,
    pub has_rewards: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPortfolioRequest {
    pub address: String,
    pub chains: Vec<Chain>,
    pub providers: Option<Vec<String>>,
    pub include_yields: Option<bool>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeFiPositionsRequest {
    pub address: String,
    pub filters: Option<DeFiPositionFilters>,
}

impl Default for DeFiPortfolioRequest {
    fn default() -> Self {
        Self {
            address: String::new(),
            chains: vec![],
            providers: None,
            include_yields: Some(true),
            currency: Some("USD".to_string()),
        }
    }
}
