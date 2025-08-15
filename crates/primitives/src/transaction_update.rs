use crate::transaction_state::TransactionState;
use crate::transaction_metadata_types::TransactionPerpetualMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionUpdate {
    pub state: TransactionState,
    pub changes: Vec<TransactionChange>,
}

impl TransactionUpdate {
    pub fn new(state: TransactionState) -> Self {
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
}
