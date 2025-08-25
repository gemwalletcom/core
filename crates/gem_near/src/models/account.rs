use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAccessKey {
    pub nonce: i64,
}
