use hex::encode;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::swap::ApprovalData;

#[deprecated(since = "1.0.0", note = "Use transaction-specific data instead")]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            call_data: Some(encode(call_data)),
            approval: None,
            gas_limit: None,
        }
    }
}
