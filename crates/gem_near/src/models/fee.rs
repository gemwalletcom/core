use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPrice {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub gas_price: u64,
}
