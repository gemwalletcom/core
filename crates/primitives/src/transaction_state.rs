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
    pub fn is_terminal(&self) -> bool {
        match self {
            Self::Confirmed | Self::Failed | Self::Reverted => true,
            Self::Pending | Self::InTransit => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_terminal() {
        assert!(TransactionState::Confirmed.is_terminal());
        assert!(TransactionState::Failed.is_terminal());
        assert!(TransactionState::Reverted.is_terminal());
        assert!(!TransactionState::Pending.is_terminal());
        assert!(!TransactionState::InTransit.is_terminal());
    }
}
