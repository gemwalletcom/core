use crate::{Delegation, DelegationValidator, UInt64};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct RedelegateData {
    pub delegation: Delegation,
    #[serde(rename = "toValidator")]
    pub to_validator: DelegationValidator,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum Resource {
    Bandwidth,
    Energy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub enum StakeType {
    Stake(DelegationValidator),
    Unstake(Delegation),
    Redelegate(RedelegateData),
    Rewards(Vec<DelegationValidator>),
    Withdraw(Delegation),
    Freeze(Resource),
    Unfreeze(Resource),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct TronVote {
    pub validator: String,
    pub count: UInt64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct TronUnfreeze {
    pub resource: Resource,
    pub amount: UInt64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub enum TronStakeData {
    Votes(Vec<TronVote>),
    Unfreeze(Vec<TronUnfreeze>),
}
