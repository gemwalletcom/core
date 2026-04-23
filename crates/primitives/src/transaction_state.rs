use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumIter, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum TransactionState {
    Pending,
    Confirmed,
    InTransit,
    Failed,
    Reverted,
}

impl TransactionState {
    pub fn is_completed(&self) -> bool {
        match self {
            Self::Pending | Self::InTransit => false,
            Self::Confirmed | Self::Failed | Self::Reverted => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_completed() {
        assert!(!TransactionState::Pending.is_completed());
        assert!(!TransactionState::InTransit.is_completed());
        assert!(TransactionState::Confirmed.is_completed());
        assert!(TransactionState::Failed.is_completed());
        assert!(TransactionState::Reverted.is_completed());
    }
}
