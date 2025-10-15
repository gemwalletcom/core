use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// Represents staking lock duration values in seconds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare]
pub struct StakeLockTime {
    /// Time required before funds become withdrawable once an undelegation is initiated.
    pub withdrawal: u64,
    /// Optional time required for a newly created stake to become active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation: Option<u64>,
}

impl StakeLockTime {
    pub const fn new(withdrawal: u64, activation: Option<u64>) -> Self {
        Self { withdrawal, activation }
    }
}

impl Default for StakeLockTime {
    fn default() -> Self {
        Self::new(0, None)
    }
}
