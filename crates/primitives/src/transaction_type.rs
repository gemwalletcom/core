use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, AsRefStr, PartialEq, EnumIter)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
#[derive(Default)]
pub enum TransactionType {
    #[default]
    Transfer,
    TransferNFT,
    Swap,
    TokenApproval,
    StakeDelegate,
    StakeUndelegate,
    StakeRewards,
    StakeRedelegate,
    StakeWithdraw,
    StakeFreeze,
    StakeUnfreeze,
    AssetActivation,
    SmartContractCall,
    PerpetualOpenPosition,
    PerpetualClosePosition,
}

impl TransactionType {
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }
}
