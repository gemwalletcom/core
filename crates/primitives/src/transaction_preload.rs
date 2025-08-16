use serde::{Deserialize, Serialize};

use crate::{UTXO};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPreloadInput {
    pub sender_address: String,
    pub destination_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPreload {
    pub block_hash: String,
    pub block_number: i64,
    pub utxos: Vec<UTXO>,
    pub sequence: i64,
    pub chain_id: String,
    pub is_destination_address_exist: bool,
}

impl Default for TransactionPreload {
    fn default() -> Self {
        Self {
            block_hash: String::new(),
            block_number: 0,
            utxos: vec![],
            sequence: 0,
            chain_id: String::new(),
            is_destination_address_exist: true,
        }
    }
}