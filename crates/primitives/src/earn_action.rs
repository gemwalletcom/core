use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[deprecated(since = "1.0.0", note = "Use EarnYieldType instead")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub enum EarnAction {
    Deposit,
    Withdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum EarnYieldType {
    Deposit { provider_id: String },
    Withdraw { provider_id: String },
}
