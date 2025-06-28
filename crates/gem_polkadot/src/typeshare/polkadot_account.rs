use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct PolkadotAccountBalance {
    pub free: String,
    pub reserved: String,
    pub nonce: String,
}