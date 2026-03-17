use primitives::swap::SwapStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidRouteRequest {
    pub from_chain: String,
    pub to_chain: String,
    pub from_token: String,
    pub to_token: String,
    pub from_amount: String,
    pub from_address: String,
    pub to_address: String,
    pub slippage_config: SlippageConfig,
    pub quote_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlippageConfig {
    pub auto_mode: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SquidRouteResponse {
    pub route: SquidRoute,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidRoute {
    pub estimate: SquidEstimate,
    #[serde(deserialize_with = "deserialize_transaction_request")]
    pub transaction_request: Option<SquidTransactionRequest>,
}

fn deserialize_transaction_request<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<SquidTransactionRequest>, D::Error> {
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        Some(v) if v.as_object().is_some_and(|m| m.contains_key("data")) => serde_json::from_value(v).map(Some).map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidEstimate {
    pub to_amount: String,
    pub estimated_route_duration: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidTransactionRequest {
    pub target: String,
    pub data: String,
    pub value: String,
    pub gas_limit: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidStatusResponse {
    pub squid_transaction_status: SquidStatus,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SquidStatus {
    Success,
    Ongoing,
    PartialSuccess,
    NeedsGas,
    NotFound,
    Refund,
}

impl SquidStatus {
    pub fn swap_status(&self) -> SwapStatus {
        match self {
            Self::Success | Self::PartialSuccess => SwapStatus::Completed,
            Self::Ongoing | Self::NeedsGas | Self::NotFound => SwapStatus::Pending,
            Self::Refund => SwapStatus::Failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_status_response() {
        let result: SquidStatusResponse = serde_json::from_str(include_str!("../../testdata/squid/status_response.json")).unwrap();
        assert_eq!(result.squid_transaction_status, SquidStatus::Success);
        assert_eq!(result.squid_transaction_status.swap_status(), SwapStatus::Completed);
    }
}
