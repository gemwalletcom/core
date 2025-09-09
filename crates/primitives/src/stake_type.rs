use crate::{Delegation, DelegationValidator};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct RedelegateData {
    pub delegation: Delegation,
    #[serde(rename = "toValidator")]
    pub to_validator: DelegationValidator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct StakeData {
    pub data: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub enum StakeType {
    Stake(DelegationValidator),
    Unstake(Delegation),
    Redelegate(RedelegateData),
    Rewards(Vec<DelegationValidator>),
    Withdraw(Delegation),
}
