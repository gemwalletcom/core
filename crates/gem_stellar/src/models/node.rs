use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarNodeStatus {
    pub ingest_latest_ledger: i32,
    pub network_passphrase: String,
}

// RPC models
#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub ingest_latest_ledger: i32,
    pub network_passphrase: String,
}
