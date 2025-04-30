use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub amount: String,
    pub src_chain: String,
    pub src_asset: String,
    pub dest_chain: String,
    pub dest_asset: String,
    pub is_vault_swap: bool,
    pub dca_enabled: bool,
    pub broker_commission_bps: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncludedFee {
    #[serde(rename = "type")]
    pub fee_type: String,
    pub chain: String,
    pub asset: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub intermediate_amount: Option<String>,
    pub egress_amount: String,
    pub recommended_slippage_tolerance_percent: f64,
    pub included_fees: Vec<IncludedFee>,
    pub low_liquidity_warning: bool,
    pub estimated_duration_seconds: f64,
    #[serde(rename = "type")]
    pub quote_type: String,
    pub deposit_amount: String,
    pub is_vault_swap: bool,
}

impl QuoteResponse {
    pub fn slippage_bps(&self) -> u32 {
        (self.recommended_slippage_tolerance_percent * 100.0) as u32
    }
}
