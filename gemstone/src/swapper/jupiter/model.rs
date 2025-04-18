use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: String,
    pub slippage_bps: u32,
    pub platform_fee_bps: u32,
    pub auto_slippage: bool,
    pub max_auto_slippage_bps: u32,
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
    pub computed_auto_slippage: Option<u32>,
    pub platform_fee: PlatformFee,
    pub price_impact_pct: String,
    pub route_plan: Vec<Route>,
    pub context_slot: i64,
    pub time_taken: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFee {
    pub amount: String,
    pub fee_bps: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub swap_info: RouteSwapInfo,
    pub percent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSwapInfo {
    pub amm_key: String,
    pub label: String,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
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
    pub error_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicSlippage {
    pub max_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDataRequest {
    pub user_public_key: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub fee_account: String,
    pub quote_response: QuoteResponse,
    pub prioritization_fee_lamports: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_slippage: Option<DynamicSlippage>,
}
