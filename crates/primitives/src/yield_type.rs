use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Chain, swap::ApprovalData};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub enum YieldType {
    Deposit,
    Withdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct YieldTransaction {
    pub chain: Chain,
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: Option<String>,
    pub approval: Option<ApprovalData>,
}
