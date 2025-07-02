use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPLatestBlock {
    pub ledger_current_index: UInt64,
}
