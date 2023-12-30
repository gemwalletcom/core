use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, AsRefStr)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum TransactionType {
    Transfer,
    Swap,
    TokenApproval,
    StakeDelegate,
    StakeUndelegate,
    StakeRewards,
}

impl Default for TransactionType {
    fn default() -> Self {
        Self::Transfer
    }
}
