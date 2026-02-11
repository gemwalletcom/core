use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Chain, Delegation, DelegationValidator, swap::ApprovalData};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub enum EarnType {
    Deposit(DelegationValidator),
    Withdraw(Delegation),
}

impl EarnType {
    pub fn validator(&self) -> &DelegationValidator {
        match self {
            EarnType::Deposit(validator) => validator,
            EarnType::Withdraw(delegation) => &delegation.validator,
        }
    }

    pub fn provider_id(&self) -> &str {
        &self.validator().id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct EarnTransaction {
    pub chain: Chain,
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: Option<String>,
    pub approval: Option<ApprovalData>,
}
