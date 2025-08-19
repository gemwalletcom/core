use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosAccount {
    #[serde(deserialize_with = "serde_serializers::deserialize_u64_from_str")]
    pub account_number: u64,
    #[serde(deserialize_with = "serde_serializers::deserialize_u64_from_str")]
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosAccountResponse<T> {
    pub account: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosInjectiveAccount {
    pub base_account: CosmosAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosBalances {
    pub balances: Vec<CosmosBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosBalance {
    pub denom: String,
    pub amount: String,
}
