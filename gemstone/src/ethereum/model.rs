use primitives::fee::FeePriority;
use serde::{Deserialize, Serialize};

pub type GemFeePriority = FeePriority;

#[uniffi::remote(Enum)]
pub enum GemFeePriority {
    Slow,
    Normal,
    Fast,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPriorityFeeRecord {
    pub priority: GemFeePriority,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct GemEthereumFeeHistory {
    pub reward: Vec<Vec<String>>,
    pub base_fee_per_gas: Vec<String>,
    pub gas_used_ratio: Vec<f64>,
}
