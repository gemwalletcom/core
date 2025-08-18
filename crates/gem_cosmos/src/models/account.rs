use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosAccount {
    pub account_number: String,
    pub sequence: String,
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
