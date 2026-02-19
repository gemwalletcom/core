use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::swap::ApprovalData;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct EarnData {
    pub contract_address: String,
    pub call_data: String,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}
