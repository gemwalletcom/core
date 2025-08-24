use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    #[serde(deserialize_with = "serde_serializers::deserialize_u64_from_str")]
    pub account_number: u64,
    #[serde(deserialize_with = "serde_serializers::deserialize_u64_from_str")]
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponse<T> {
    pub account: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectiveAccount {
    pub base_account: Account,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balances {
    pub balances: Vec<Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub denom: String,
    pub amount: String,
}
