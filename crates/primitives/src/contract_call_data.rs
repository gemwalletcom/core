use crate::swap::ApprovalData;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct ContractCallData {
    pub contract_address: String,
    pub call_data: String,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}
