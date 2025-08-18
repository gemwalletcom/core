use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasFee {
    pub deprioritized_gas_estimate: u64,
    pub gas_estimate: u64,
    pub prioritized_gas_estimate: u64,
}