use crate::fee::FeePriority;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityFeeValue {
    pub priority: FeePriority,
    pub value: BigInt,
}
