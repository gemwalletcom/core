use crate::fee::FeePriority;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
pub struct PriorityFeeValue {
    pub priority: FeePriority,
    pub value: BigInt,
}