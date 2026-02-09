use crate::{EarnPosition, EarnProvider};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct RedelegateData {
    pub position: EarnPosition,
    pub to_provider: EarnProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub enum Resource {
    Bandwidth,
    Energy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub enum FreezeType {
    Freeze,
    Unfreeze,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct FreezeData {
    #[serde(rename = "freezeType")]
    pub freeze_type: FreezeType,
    pub resource: Resource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub enum StakeType {
    Stake(EarnProvider),
    Unstake(EarnPosition),
    Redelegate(RedelegateData),
    Rewards(Vec<EarnProvider>),
    Withdraw(EarnPosition),
    Freeze(FreezeData),
}
