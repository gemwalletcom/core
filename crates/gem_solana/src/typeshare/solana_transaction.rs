use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::Int;

#[typeshare(swift = "Sendable")]
pub struct SolanaTransaction {
    pub meta: SolanaTransactionMeta,
    pub slot: Int,
}

#[typeshare(swift = "Sendable")]
pub struct SolanaTransactionMeta {
    pub err: Option<SolanaTransactionError>,
}

#[typeshare(swift = "Sendable")]
pub struct SolanaTransactionError {}

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaInstruction {
    pub program_id: String,
    pub accounts: Vec<SolanaAccountMeta>,
    pub data: String,
}

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}
