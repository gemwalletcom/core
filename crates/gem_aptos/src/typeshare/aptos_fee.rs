use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosGasFee {
    pub deprioritized_gas_estimate: i32,
    pub gas_estimate: i32,
    pub prioritized_gas_estimate: i32,
}