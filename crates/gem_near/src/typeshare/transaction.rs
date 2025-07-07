use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct NearBroadcastResult {
    pub final_execution_status: String,
    pub transaction: NearBroadcastTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct NearBroadcastTransaction {
    pub hash: String,
}
