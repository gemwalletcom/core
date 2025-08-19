use serde::{Deserialize, Serialize};

use super::UInt64;

#[derive(Serialize, Deserialize)]
pub struct SolanaTransaction {
    pub meta: SolanaTransactionMeta,
    pub slot: UInt64,
}

#[derive(Serialize, Deserialize)]
pub struct SolanaTransactionMeta {
    pub err: Option<SolanaTransactionError>,
}

#[derive(Serialize, Deserialize)]
pub struct SolanaTransactionError {}
