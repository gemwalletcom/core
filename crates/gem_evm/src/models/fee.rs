use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthereumFeeHistory {
    pub reward: Vec<Vec<String>>,
    pub base_fee_per_gas: Vec<String>,
    pub gas_used_ratio: Vec<f64>,
    pub oldest_block: String,
}
