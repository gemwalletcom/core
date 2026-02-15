use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: String,
    pub slippage_bps: u32,
    pub platform_fee_bps: u32,
    pub instruction_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u32,
    pub platform_fee: Value,
    pub price_impact_pct: String,
    pub route_plan: Value,
    pub context_slot: i64,
    pub time_taken: f64,
    pub instruction_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDataResponse {
    pub swap_transaction: String,
    pub simulation_error: Option<SimulationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationError {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDataRequest {
    pub user_public_key: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub fee_account: String,
    pub quote_response: QuoteResponse,
    pub prioritization_fee_lamports: i64,
}
