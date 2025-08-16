use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::block::CardanoBlock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct CardanoTransactionBroadcast {
    pub submit_transaction: Option<CardanoSubmitTransactionHash>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct CardanoSubmitTransactionHash {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct CardanoTransactions {
    pub transactions: Vec<CardanoTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct CardanoTransaction {
    pub fee: String,
    pub block: CardanoBlock,
}
