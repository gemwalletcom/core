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
pub struct YieldData {
    pub provider_name: String,
    pub contract_address: String,
    pub call_data: String,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct EvmYieldData {
    pub contract_address: String,
    pub call_data: String,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}
