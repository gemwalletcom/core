use crate::models::rpc::{Info, Parsed, ValueResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteAccounts {
    pub current: Vec<VoteAccount>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteAccount {
    pub vote_pubkey: String,
    pub node_pubkey: String,
    pub commission: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub blockhash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockhash {
    pub blockhash: String,
}

pub type LatestBlockhash = ValueResult<Blockhash>;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpochInfo {
    pub epoch: u64,
    pub slots_in_epoch: u64,
    pub slot_index: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorConfig {
    pub pubkey: String,
    pub account: ValidatorConfigAccount,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorConfigAccount {
    pub data: Parsed<Info<ValidatorConfigInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorConfigInfo {
    pub name: String,
    pub config_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InflationRate {
    pub validator: f64,
}
