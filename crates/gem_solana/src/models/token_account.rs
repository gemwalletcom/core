use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, deserialize_u64_from_str};

use super::UInt64;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaTokenAccountPubkey {
    pub pubkey: String,
}

// accounts
pub struct SolanaStakeAccount {
    pub account: SolanaAccount<SolanaAccountParsed<SolanaAccountParsedInfo<SolanaStakeInfo>>>,
    pub pubkey: String,
}

pub struct SolanaTokenAccount {
    pub account: SolanaAccount<SolanaAccountParsed<SolanaAccountParsedInfo<SolanaTokenInfo>>>,
    pub pubkey: String,
}

// parsed data

pub struct SolanaAccount<T> {
    pub lamports: UInt64,
    pub space: UInt64,
    pub owner: String,
    pub data: T,
}

pub struct SolanaAccountParsed<T> {
    pub parsed: T,
}

pub struct SolanaAccountParsedInfo<T> {
    pub info: T,
}

// parsed data: stake
pub struct SolanaStakeInfo {
    pub stake: SolanaStake,
    pub meta: SolanaRentExemptReserve,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaRentExemptReserve {
    pub rent_exempt_reserve: String,
}

pub struct SolanaStake {
    pub delegation: SolanaStakeDelegation,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaStakeDelegation {
    pub voter: String,
    pub stake: String,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub activation_epoch: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub deactivation_epoch: u64,
}

// parsed data: token
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaTokenInfo {
    pub token_amount: SolanaTokenAmount,
}

#[derive(Serialize, Deserialize)]
pub struct SolanaTokenAmount {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
}
