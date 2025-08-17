use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearAccount {
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearAccountAccessKey {
    pub nonce: i64,
}
