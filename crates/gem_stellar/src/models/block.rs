#[cfg(feature = "rpc")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub closed_at: String,
    pub sequence: i64,
    pub base_fee_in_stroops: i64,
}
