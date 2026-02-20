use serde::{Deserialize, Serialize};

pub const DEPOSIT_TYPE_ORIGIN: &str = "ORIGIN_CHAIN";
pub const RECIPIENT_TYPE_DESTINATION: &str = "DESTINATION_CHAIN";
pub const DEFAULT_WAIT_TIME_MS: u32 = 1_024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppFee {
    pub recipient: String,
    pub fee: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub origin_asset: String,
    pub destination_asset: String,
    pub amount: String,
    pub referral: String,
    pub recipient: String,
    pub swap_type: SwapType,
    pub slippage_tolerance: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_fees: Option<Vec<AppFee>>,
    pub deposit_type: String,
    pub refund_to: String,
    pub refund_type: String,
    pub recipient_type: String,
    pub deadline: String,
    pub quote_waiting_time_ms: Option<u32>,
    pub dry: bool,
    pub deposit_mode: DepositMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SwapType {
    ExactInput,
    FlexInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepositMode {
    #[default]
    Simple,
    Memo,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub quote_request: QuoteRequest,
    pub quote: Quote,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuoteResponseError {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum QuoteResponseResult {
    Ok(Box<QuoteResponse>),
    Err(QuoteResponseError),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub deposit_address: Option<String>,
    pub deposit_memo: Option<String>,
    pub deposit_mode: Option<DepositMode>,
    pub amount_in: String,
    pub amount_in_formatted: String,
    pub min_amount_in: String,
    pub amount_out: String,
    pub amount_out_formatted: String,
    pub min_amount_out: String,
    pub deadline: Option<String>,
    pub time_when_inactive: Option<String>,
    pub time_estimate: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionStatus {
    pub quote_response: Option<QuoteResponse>,
    pub status: String,
    pub updated_at: String,
    pub swap_details: Option<SwapDetails>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SwapDetails {
    pub amount_in: Option<String>,
    pub amount_out: Option<String>,
    pub origin_chain_tx_hashes: Vec<TransactionDetails>,
    pub destination_chain_tx_hashes: Vec<TransactionDetails>,
    pub refunded_amount: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetails {
    pub hash: String,
    pub explorer_url: Option<String>,
}
