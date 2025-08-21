use crate::transaction_metadata_types::TransactionPerpetualMetadata;
use crate::transaction_state::TransactionState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionUpdate {
    pub state: TransactionState,
    pub changes: Vec<TransactionChange>,
}

impl TransactionUpdate {
    pub fn new(state: TransactionState, changes: Vec<TransactionChange>) -> Self {
        Self { state, changes }
    }

    pub fn new_state(state: TransactionState) -> Self {
        Self { state, changes: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionChange {
    HashChange { old: String, new: String },
    Metadata(TransactionMetadata),
    BlockNumber(String),
    NetworkFee(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionMetadata {
    Perpetual(TransactionPerpetualMetadata),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionStateRequest {
    pub id: String,
    pub sender_address: String,
    pub created_at: i64,
    pub block_number: i64,
}
