use serde::{Deserialize, Serialize};

use primitives::swap::SwapStatus;
use serde_serializers::deserialize_string_from_value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayQuoteRequest {
    pub user: String,
    pub origin_chain_id: u64,
    pub destination_chain_id: u64,
    pub origin_currency: String,
    pub destination_currency: String,
    pub amount: String,
    pub recipient: String,
    pub trade_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_tolerance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub app_fees: Vec<RelayAppFee>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelayAppFee {
    pub recipient: String,
    pub fee: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayQuoteResponse {
    pub steps: Vec<Step>,
    pub details: QuoteDetails,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub id: String,
    pub kind: String,
    pub items: Option<Vec<StepItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepItem {
    pub data: Option<StepData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepData {
    pub to: Option<String>,
    pub data: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_from_value")]
    pub value: String,
    pub instructions: Option<serde_json::Value>,
    pub address_lookup_table_addresses: Option<Vec<String>>,
    pub psbt: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayInstruction {
    pub program_id: String,
    pub keys: Vec<RelayAccountMeta>,
    pub data: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDetails {
    pub currency_out: CurrencyAmount,
    pub time_estimate: Option<f64>,
}

impl QuoteDetails {
    pub fn time_estimate_u32(&self) -> Option<u32> {
        let value = self.time_estimate?;
        if !value.is_finite() || value < 0.0 || value > u32::MAX as f64 {
            return None;
        }
        Some(value.ceil() as u32)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyAmount {
    pub amount: String,
    pub minimum_amount: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelayStatus {
    Pending,
    Waiting,
    Success,
    Completed,
    Failed,
    Refunded,
    #[serde(other)]
    Unknown,
}

impl RelayStatus {
    pub fn into_swap_status(self) -> SwapStatus {
        match self {
            RelayStatus::Pending | RelayStatus::Waiting | RelayStatus::Unknown => SwapStatus::Pending,
            RelayStatus::Success | RelayStatus::Completed => SwapStatus::Completed,
            RelayStatus::Failed | RelayStatus::Refunded => SwapStatus::Failed,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequestsResponse {
    pub requests: Vec<RelayRequest>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequest {
    pub status: RelayStatus,
    pub metadata: Option<RelayRequestMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequestMetadata {
    pub currency_in: Option<RelayCurrencyDetail>,
    pub currency_out: Option<RelayCurrencyDetail>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayCurrencyDetail {
    pub currency: String,
    pub chain_id: u64,
    pub amount: Option<String>,
}
