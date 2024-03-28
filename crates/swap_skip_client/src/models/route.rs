use crate::models::fee::EstimatedFee;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteRequest {
    pub amount_in: String,
    pub source_asset_denom: String,
    pub source_asset_chain_id: String,
    pub dest_asset_denom: String,
    pub dest_asset_chain_id: String,
    pub cumulative_affiliate_fee_bps: String,
    pub allow_multi_tx: bool,
    pub client_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteResponse {
    pub source_asset_denom: String,
    pub source_asset_chain_id: String,
    pub dest_asset_denom: String,
    pub dest_asset_chain_id: String,
    pub amount_in: String,
    pub amount_out: String,
    pub operations: serde_json::Value,
    pub does_swap: bool,
    pub estimated_amount_out: String,
    pub txs_required: i64,
    pub usd_amount_in: String,
    pub usd_amount_out: String,
    pub swap_price_impact_percent: String,
    pub estimated_fees: Vec<EstimatedFee>,
}
