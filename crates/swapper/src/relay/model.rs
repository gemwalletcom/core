use gem_evm::address::ethereum_address_checksum;
use primitives::swap::{SwapMode, SwapStatus};
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_string_from_value;

pub fn relay_trade_type(mode: &SwapMode) -> &'static str {
    match mode {
        SwapMode::ExactIn => "EXACT_INPUT",
        SwapMode::ExactOut => "EXACT_OUTPUT",
    }
}

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
    pub referrer: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub app_fees: Vec<RelayAppFee>,
    pub max_route_length: u32,
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
    pub fees: Option<RelayFees>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayFees {
    pub gas: Option<RelayFeeAmount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayFeeAmount {
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub id: String,
    pub kind: String,
    pub items: Option<Vec<StepItem>>,
}

impl Step {
    pub fn step_data(&self) -> Option<&StepData> {
        self.items.as_ref()?.first()?.data.as_ref()
    }
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
    pub psbt: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDetails {
    pub currency_out: CurrencyAmount,
    pub time_estimate: Option<f64>,
    pub swap_impact: Option<SwapImpact>,
}

impl QuoteDetails {
    pub fn time_estimate_u32(&self) -> Option<u32> {
        let value = self.time_estimate?;
        if !value.is_finite() || value < 0.0 || value > u32::MAX as f64 {
            return None;
        }
        Some(value.ceil() as u32)
    }

    pub fn slippage_bps(&self) -> Option<u32> {
        let percent: f64 = self.swap_impact.as_ref()?.percent.as_ref()?.parse().ok()?;
        Some((percent.abs() * 100.0) as u32)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapImpact {
    pub percent: Option<String>,
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
    Failure,
    Refunded,
    #[serde(other)]
    Unknown,
}

impl RelayStatus {
    pub fn into_swap_status(self) -> SwapStatus {
        match self {
            RelayStatus::Pending | RelayStatus::Waiting | RelayStatus::Unknown => SwapStatus::Pending,
            RelayStatus::Success | RelayStatus::Completed => SwapStatus::Completed,
            RelayStatus::Failed | RelayStatus::Failure | RelayStatus::Refunded => SwapStatus::Failed,
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
    pub data: Option<RelayRequestData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequestData {
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
    pub currency: RelayCurrency,
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayCurrency {
    pub chain_id: u64,
    pub address: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainsResponse {
    pub chains: Vec<RelayChainInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainInfo {
    #[serde(default)]
    pub solver_addresses: Vec<String>,
}

impl RelayChainsResponse {
    pub fn solver_addresses(&self) -> Vec<String> {
        self.chains
            .iter()
            .flat_map(|c| &c.solver_addresses)
            .map(|addr| ethereum_address_checksum(addr).unwrap_or_else(|_| addr.clone()))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
