use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::UInt64;

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosGasFee {
    pub deprioritized_gas_estimate: UInt64,
    pub gas_estimate: UInt64,
    pub prioritized_gas_estimate: UInt64,
}
