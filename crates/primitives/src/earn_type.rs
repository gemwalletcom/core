use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Delegation, DelegationValidator};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub enum EarnType {
    Deposit(DelegationValidator),
    Withdraw(Delegation),
}
