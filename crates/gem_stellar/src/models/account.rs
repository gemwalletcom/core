use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarAccount {
    pub sequence: String,
    pub balances: Vec<StellarBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarBalance {
    pub balance: String,
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
}
