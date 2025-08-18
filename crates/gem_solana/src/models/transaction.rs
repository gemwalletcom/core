use serde::{Deserialize, Serialize};

use super::UInt64;

pub struct SolanaTransaction {
    pub meta: SolanaTransactionMeta,
    pub slot: UInt64,
}

pub struct SolanaTransactionMeta {
    pub err: Option<SolanaTransactionError>,
}

pub struct SolanaTransactionError {}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaInstruction {
    pub program_id: String,
    pub accounts: Vec<SolanaAccountMeta>,
    pub data: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}
