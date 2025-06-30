use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::typeshare::UInt64;

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaValidators {
    pub current: Vec<SolanaValidator>,
}

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaValidator {
    pub vote_pubkey: String,
    pub commission: i32,
    pub epoch_vote_account: bool,
}

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaEpoch {
    pub epoch: UInt64,
    pub slot_index: UInt64,
    pub slots_in_epoch: UInt64,
}
