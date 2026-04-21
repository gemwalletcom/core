use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{SwapProvider, TransactionState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
    pub is_unlimited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SwapQuoteDataType {
    Contract,
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapQuoteData {
    pub to: String,
    pub data_type: SwapQuoteDataType,
    pub value: String,
    pub data: String,
    pub memo: Option<String>,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}

impl SwapQuoteData {
    pub fn gas_limit_as_u32(&self) -> Result<u32, &'static str> {
        self.gas_limit.as_ref().ok_or("gas_limit is required")?.parse().map_err(|_| "invalid gas_limit")
    }

    pub fn new_contract(to: String, value: String, data: String, approval: Option<ApprovalData>, gas_limit: Option<String>) -> Self {
        Self {
            to,
            data_type: SwapQuoteDataType::Contract,
            value,
            data,
            memo: None,
            approval,
            gas_limit,
        }
    }

    pub fn new_tranfer(to: String, value: String, memo: Option<String>) -> Self {
        Self {
            to,
            data_type: SwapQuoteDataType::Transfer,
            value,
            data: "".to_string(),
            memo,
            approval: None,
            gas_limit: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapData {
    pub quote: SwapQuote,
    pub data: SwapQuoteData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapQuote {
    pub from_address: String,
    pub from_value: String,
    pub to_address: String,
    pub to_value: String,
    pub provider_data: SwapProviderData,
    pub slippage_bps: u32,
    pub eta_in_seconds: Option<u32>,
    pub use_max_amount: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapProviderData {
    pub provider: SwapProvider,
    pub name: String,
    pub protocol_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum SwapStatus {
    Pending,
    InTransit,
    Completed,
    Failed,
}

impl SwapStatus {
    pub fn transaction_state(&self) -> TransactionState {
        match self {
            SwapStatus::Pending => TransactionState::Pending,
            SwapStatus::InTransit => TransactionState::InTransit,
            SwapStatus::Completed => TransactionState::Confirmed,
            SwapStatus::Failed => TransactionState::Failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_status_transaction_state() {
        assert_eq!(SwapStatus::Pending.transaction_state(), TransactionState::Pending);
        assert_eq!(SwapStatus::InTransit.transaction_state(), TransactionState::InTransit);
        assert_eq!(SwapStatus::Completed.transaction_state(), TransactionState::Confirmed);
        assert_eq!(SwapStatus::Failed.transaction_state(), TransactionState::Failed);
    }
}
