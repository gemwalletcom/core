use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_fee_bps: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub slippage_bps: u32,
    pub price_impact_pct: String,
    pub route_plan: Vec<RoutePlan>,
    pub context_slot: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_amount_threshold: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_out_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_fee: Option<PlatformFee>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simulated_compute_units: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFee {
    pub amount: String,
    pub fee_bps: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segmenter_fee_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segmenter_fee_pct: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlan {
    pub venue: String,
    pub market_key: String,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub input_mint_decimals: u8,
    pub output_mint_decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDataResponse {
    pub swap_transaction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_unit_price: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prioritization_fee_lamports: Option<i64>,
}
