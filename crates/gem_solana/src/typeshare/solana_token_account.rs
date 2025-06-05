use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::Int;

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaTokenAccountPubkey {
    pub pubkey: String,
}

// accounts
#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
pub struct SolanaStakeAccount {
    pub account: SolanaAccount<SolanaAccountParsed<SolanaAccountParsedInfo<SolanaStakeInfo>>>,
    pub pubkey: String,
}

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
pub struct SolanaTokenAccount {
    pub account: SolanaAccount<SolanaAccountParsed<SolanaAccountParsedInfo<SolanaTokenInfo>>>,
    pub pubkey: String,
}

// parsed data

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
pub struct SolanaAccount<T> {
    pub lamports: Int,
    pub space: Int,
    pub owner: String,
    pub data: T,
}

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
pub struct SolanaAccountParsed<T> {
    pub parsed: T,
}

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
pub struct SolanaAccountParsedInfo<T> {
    pub info: T,
}

// parsed data: stake
#[typeshare(swift = "Sendable")]
pub struct SolanaStakeInfo {
    pub stake: SolanaStake,
    pub meta: SolanaRentExemptReserve,
}

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaRentExemptReserve {
    pub rent_exempt_reserve: String,
}

#[typeshare(swift = "Sendable")]
pub struct SolanaStake {
    pub delegation: SolanaStakeDelegation,
}

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaStakeDelegation {
    pub voter: String,
    pub stake: String,
    pub activation_epoch: String,
    pub deactivation_epoch: String,
}

// parsed data: token
#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaTokenInfo {
    pub token_amount: SolanaTokenAmount,
}

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
pub struct SolanaTokenAmount {
    pub amount: String,
}
