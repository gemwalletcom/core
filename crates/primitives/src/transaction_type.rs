use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, AsRefStr)]
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
    AssetActivation,
    SmartContractCall,
}

impl Default for TransactionType {
    fn default() -> Self {
        Self::Transfer
    }
}
