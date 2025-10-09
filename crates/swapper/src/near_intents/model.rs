use serde::{Deserialize, Serialize};

pub const DEFAULT_REFERRAL: &str = "gemwallet";
pub const DEPOSIT_TYPE_ORIGIN: &str = "ORIGIN_CHAIN";
pub const RECIPIENT_TYPE_DESTINATION: &str = "DESTINATION_CHAIN";
pub const DEFAULT_WAIT_TIME_MS: u32 = 1_000;

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
    #[serde(default = "default_referral")]
    pub referral: String,
    pub recipient: String,
    pub swap_type: SwapType,
    #[serde(default = "default_slippage_tolerance")]
    pub slippage_tolerance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_fees: Option<Vec<AppFee>>,
    #[serde(default = "default_deposit_type")]
    pub deposit_type: String,
    #[serde(default)]
    pub refund_to: String,
    #[serde(default = "default_refund_type")]
    pub refund_type: String,
    #[serde(default = "default_recipient_type")]
    pub recipient_type: String,
    #[serde(default)]
    pub deadline: String,
    #[serde(default = "default_quote_waiting_time_ms")]
    pub quote_waiting_time_ms: u32,
    #[serde(default)]
    pub dry: bool,
    #[serde(default)]
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
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub deposit_address: Option<String>,
    pub deposit_memo: Option<String>,
    #[serde(default)]
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
    #[serde(default)]
    pub virtual_chain_recipient: Option<String>,
    #[serde(default)]
    pub virtual_chain_refund_recipient: Option<String>,
    #[serde(default)]
    pub custom_recipient_msg: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionStatus {
    #[serde(default)]
    pub quote_response: Option<QuoteResponse>,
    pub status: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub swap_details: Option<SwapDetails>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SwapDetails {
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
    pub origin_chain_tx_hashes: Vec<TransactionDetails>,
    #[serde(default)]
    pub destination_chain_tx_hashes: Vec<TransactionDetails>,
    #[serde(default)]
    pub refunded_amount: Option<String>,
    #[serde(default)]
    pub refunded_amount_formatted: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetails {
    pub hash: String,
    #[serde(default)]
    pub explorer_url: Option<String>,
}

fn default_referral() -> String {
    DEFAULT_REFERRAL.to_string()
}

fn default_deposit_type() -> String {
    DEPOSIT_TYPE_ORIGIN.to_string()
}

fn default_refund_type() -> String {
    DEPOSIT_TYPE_ORIGIN.to_string()
}

fn default_recipient_type() -> String {
    RECIPIENT_TYPE_DESTINATION.to_string()
}

fn default_slippage_tolerance() -> f64 {
    0.0
}

fn default_quote_waiting_time_ms() -> u32 {
    DEFAULT_WAIT_TIME_MS
}
