use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::Int;

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
    pub epoch: Int,
    pub slot_index: Int,
    pub slots_in_epoch: Int,
}
