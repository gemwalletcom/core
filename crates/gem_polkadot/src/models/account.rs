use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotAccountBalance {
    pub free: String,
    pub reserved: String,
    pub nonce: String,
}
