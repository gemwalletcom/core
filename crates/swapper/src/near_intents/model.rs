use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsAppFee {
    pub recipient: String,
    pub fee: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsQuoteRequest {
    pub origin_asset: String,
    pub destination_asset: String,
    pub amount: String,
    pub recipient: String,
    pub swap_type: SwapType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_tolerance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_fees: Option<Vec<NearIntentsAppFee>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deposit_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_waiting_time_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SwapType {
    ExactInput,
    FlexInput,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsQuoteResponse {
    pub quote_request: NearIntentsQuoteRequest,
    pub quote: NearIntentsQuote,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsQuote {
    pub deposit_address: Option<String>,
    pub deposit_memo: Option<String>,
    pub amount_in: String,
    pub amount_in_formatted: String,
    pub min_amount_in: String,
    pub amount_out: String,
    pub amount_out_formatted: String,
    pub min_amount_out: String,
    pub deadline: Option<String>,
    pub time_when_inactive: Option<String>,
    pub time_estimate: u32,
    #[serde(default)]
    pub virtual_chain_recipient: Option<String>,
    #[serde(default)]
    pub virtual_chain_refund_recipient: Option<String>,
    #[serde(default)]
    pub custom_recipient_msg: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsExecutionStatus {
    #[serde(default)]
    pub quote_response: Option<NearIntentsQuoteResponse>,
    pub status: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub swap_details: Option<NearIntentsSwapDetails>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsSwapDetails {
    #[serde(default)]
    pub intent_hashes: Vec<String>,
    #[serde(default)]
    pub near_tx_hashes: Vec<String>,
    #[serde(default)]
    pub amount_in: Option<String>,
    #[serde(default)]
    pub amount_in_formatted: Option<String>,
    #[serde(default)]
    pub amount_out: Option<String>,
    #[serde(default)]
    pub amount_out_formatted: Option<String>,
    #[serde(default)]
    pub slippage: Option<u32>,
    #[serde(default)]
    pub origin_chain_tx_hashes: Vec<NearIntentsTransactionDetails>,
    #[serde(default)]
    pub destination_chain_tx_hashes: Vec<NearIntentsTransactionDetails>,
    #[serde(default)]
    pub refunded_amount: Option<String>,
    #[serde(default)]
    pub refunded_amount_formatted: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsTransactionDetails {
    pub hash: String,
    #[serde(default)]
    pub explorer_url: Option<String>,
}
