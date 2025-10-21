use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, AsRefStr, PartialEq, EnumIter)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum TransactionType {
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
    PerpetualModify,
}

impl Default for TransactionType {
    fn default() -> Self {
        Self::Transfer
    }
}

impl TransactionType {
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }
}
