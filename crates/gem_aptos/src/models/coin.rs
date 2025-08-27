use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinData {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinStore {
    pub coin: Coin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinInfo {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}
