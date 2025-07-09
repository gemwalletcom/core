use crate::{AssetId, Chain};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPortfolio {
    pub positions: Vec<DeFiPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPosition {
    pub address: String,
    pub chain: String,
    pub protocol: DeFiProtocol,
    pub position_type: String,
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
    pub asset_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPositionFilters {
    pub position_types: Vec<String>,
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
