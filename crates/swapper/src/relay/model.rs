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
    pub referrer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_to: Option<String>,
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
    #[serde(default)]
    pub items: Vec<StepItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepItem {
    pub data: Option<StepData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepData {
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub data: String,
    #[serde(default, deserialize_with = "deserialize_string_from_value")]
    pub value: String,
    #[serde(default)]
    pub instructions: Option<serde_json::Value>,
    #[serde(default)]
    pub psbt: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDetails {
    pub currency_out: CurrencyAmount,
    #[serde(default)]
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
            RelayStatus::Failed => SwapStatus::Failed,
            RelayStatus::Refunded => SwapStatus::Refunded,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayStatusResponse {
    pub status: RelayStatus,
    pub out_tx_hashes: Option<Vec<String>>,
    pub destination_chain_id: Option<u64>,
}
