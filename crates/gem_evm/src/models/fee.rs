use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_bigint_vec_from_hex_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthereumFeeHistory {
    pub reward: Vec<Vec<String>>,
    #[serde(deserialize_with = "deserialize_bigint_vec_from_hex_str")]
    pub base_fee_per_gas: Vec<BigInt>,
    pub gas_used_ratio: Vec<f64>,
    pub oldest_block: String,
}
