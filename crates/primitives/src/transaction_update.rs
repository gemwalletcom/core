use crate::transaction_state::TransactionState;
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
pub enum TransactionChange {}
