use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::swap::ApprovalData;

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
