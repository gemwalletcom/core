use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinAccount {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub balance: BigUint,
}
