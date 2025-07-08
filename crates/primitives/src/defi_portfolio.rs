use crate::{AssetId, Chain};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPortfolio {
    pub positions: Vec<DeFiPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPosition {
    pub address: String,
    pub chain: String,
    pub protocol: DeFiProtocol,
    pub position_type: DeFiPositionType,
    pub name: String,
    pub stats: PositionStats,
    pub assets: Vec<DeFiAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiProtocol {
    pub id: String,
    pub name: String,
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
    pub asset_value_usd: BigDecimal,
    pub debt_value_usd: BigDecimal,
    pub net_value_usd: BigDecimal,
    pub health_ratio: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiAsset {
    pub asset_id: AssetId,
    pub amount: BigDecimal,
    pub value_usd: BigDecimal,
    pub asset_type: DeFiAssetType,
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
pub struct DeFiPositionFilters {
    pub position_types: Vec<DeFiPositionType>,
    pub chains: Vec<Chain>,
    pub protocols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPortfolioRequest {
    pub address: String,
    pub chains: Vec<Chain>,
    pub providers: Vec<String>,
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
            providers: vec![],
        }
    }
}
