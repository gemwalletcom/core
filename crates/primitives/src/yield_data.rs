use hex::encode;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::swap::ApprovalData;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct YieldData {
    pub contract_address: Option<String>,
    pub call_data: Option<String>,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}

impl YieldData {
    pub fn stake(contract_address: String, call_data: &[u8]) -> Self {
        Self {
            contract_address: Some(contract_address),
            call_data: Some(encode(call_data)),
            approval: None,
            gas_limit: None,
        }
    }
}
