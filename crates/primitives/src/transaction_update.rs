use crate::chain::Chain;
use crate::transaction_metadata_types::{TransactionPerpetualMetadata, TransactionSwapMetadata};
use crate::transaction_state::TransactionState;
use num_bigint::BigInt;
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
    NetworkFee(BigInt),
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

impl TransactionStateRequest {
    pub fn new_id(id: String) -> Self {
        Self {
            id,
            sender_address: String::new(),
            created_at: 0,
            block_number: 0,
        }
    }

    pub fn with_block_number(mut self, block_number: i64) -> Self {
        self.block_number = block_number;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionStateInput {
    pub chain: Chain,
    pub hash: String,
    pub from: String,
    pub created_at_secs: i64,
    pub block_number: i64,
    pub swap_metadata: Option<TransactionSwapMetadata>,
}

impl From<&TransactionStateInput> for TransactionStateRequest {
    fn from(input: &TransactionStateInput) -> Self {
        Self {
            id: input.hash.clone(),
            sender_address: input.from.clone(),
            created_at: input.created_at_secs,
            block_number: input.block_number,
        }
    }
}

#[cfg(test)]
mod tests_state_input {
    use super::*;

    #[test]
    fn test_transaction_state_input_to_request() {
        let input = TransactionStateInput {
            chain: Chain::Ethereum,
            hash: "0xabc".into(),
            from: "0xfrom".into(),
            created_at_secs: 1_700_000_000,
            block_number: 18_430_221,
            swap_metadata: None,
        };

        let request: TransactionStateRequest = (&input).into();

        assert_eq!(request.id, "0xabc");
        assert_eq!(request.sender_address, "0xfrom");
        assert_eq!(request.created_at, 1_700_000_000);
        assert_eq!(request.block_number, 18_430_221);
    }
}
