use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarAccount {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub sequence: u64,
    pub balances: Vec<StellarBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarBalance {
    pub balance: String,
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
}

// RPC models
#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Account {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub sequence: u64,
    pub balances: Vec<Balance>,
}

#[cfg(feature = "rpc")]

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: String,
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
}
