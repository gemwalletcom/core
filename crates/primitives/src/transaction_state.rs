use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/*
Pending: The transaction has been initiated but is still pending confirmation. It has been broadcasted to the network but has not yet been included in a block.
Confirmed: The transaction has been included in a block and is considered confirmed. At this stage, it is considered final and irreversible.
Failed: The transaction has encountered an error or has been rejected for some reason. It did not succeed in completing its intended operation.
Reverted: The transaction was executed but later reverted due to an error or a specific condition. In this case, the transaction is considered unsuccessful, and any changes it made to the state of the system are rolled back.
*/

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum TransactionState {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

impl TransactionState {
    pub fn new(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "confirmed" => Some(Self::Confirmed),
            "failed" => Some(Self::Failed),
            "reverted" => Some(Self::Reverted),
            _ => None,
        }
    }
}

impl std::fmt::Display for TransactionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Pending => "pending".to_string(),
            Self::Confirmed => "confirmed".to_string(),
            Self::Failed => "failed".to_string(),
            Self::Reverted => "reverted".to_string(),
        };
        write!(f, "{str}")
    }
}
