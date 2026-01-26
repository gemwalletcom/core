use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::swap::ApprovalData;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub enum YieldAction {
    Deposit,
    Withdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct EarnData {
    pub provider: Option<String>,
    pub contract_address: Option<String>,
    pub call_data: Option<String>,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}

impl EarnData {
    pub fn stake(contract_address: String, call_data: &[u8]) -> Self {
        Self {
            provider: None,
            contract_address: Some(contract_address),
            call_data: if call_data.is_empty() { None } else { Some(hex::encode(call_data)) },
            approval: None,
            gas_limit: None,
        }
    }

    pub fn yield_data(provider: String, contract_address: String, call_data: String, approval: Option<ApprovalData>, gas_limit: Option<String>) -> Self {
        Self {
            provider: Some(provider),
            contract_address: Some(contract_address),
            call_data: Some(call_data),
            approval,
            gas_limit,
        }
    }
}
