pub use crate::config::evm_chain::EVMHistoryRewardPercentiles;
use primitives::fee::FeePriority;
use serde::{Deserialize, Serialize};

type GemFeePriority = FeePriority;

#[uniffi::remote(Enum)]
pub enum GemFeePriority {
    Slow,
    Normal,
    Fast,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemBasePriorityFees {
    pub base_fee: String,
    pub priority_fees: Vec<GemPriorityFeeRecord>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPriorityFeeRecord {
    pub priority: GemFeePriority,
    pub value: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct EvmFeeParams {
    pub history_blocks: u64,
    pub reward_percentiles: EVMHistoryRewardPercentiles,
    pub min_priority_fee: String,
    pub chain: primitives::Chain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GemEthereumFeeHistory {
    pub reward: Vec<Vec<String>>,
    pub base_fee_per_gas: Vec<String>,
    pub gas_used_ratio: Vec<f64>,
}
