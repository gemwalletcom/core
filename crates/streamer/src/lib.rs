pub mod stream_producer;
pub mod stream_reader;

use primitives::{Chain, Transaction};
use serde::{Deserialize, Serialize};
pub use stream_producer::StreamProducer;
pub use stream_reader::StreamReader;

use std::fmt;

#[derive(Debug, Clone)]
pub enum QueueName {
    Transactions,
}

impl fmt::Display for QueueName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueName::Transactions => write!(f, "transactions"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsPayload {
    pub chain: Chain,
    pub blocks: Vec<i32>,
    pub transactions: Vec<Transaction>,
}

impl TransactionsPayload {
    pub fn new(chain: Chain, blocks: Vec<i32>, transactions: Vec<Transaction>) -> Self {
        Self { chain, blocks, transactions }
    }
}
