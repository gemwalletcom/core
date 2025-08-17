use serde::{Deserialize, Serialize};

use super::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRPLatestBlock {
    pub ledger_current_index: UInt64,
}
