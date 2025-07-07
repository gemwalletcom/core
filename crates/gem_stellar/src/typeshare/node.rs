use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct StellarNodeStatus {
    pub ingest_latest_ledger: i32,
    pub network_passphrase: String,
}
