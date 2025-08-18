use serde::{Deserialize, Serialize};

use crate::UTXO;

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
    pub sequence: u64,
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

impl TransactionPreload {
    pub fn builder() -> TransactionPreloadBuilder {
        TransactionPreloadBuilder::default()
    }
}

#[derive(Default)]
pub struct TransactionPreloadBuilder {
    block_hash: String,
    block_number: i64,
    utxos: Vec<UTXO>,
    sequence: u64,
    chain_id: String,
    is_destination_address_exist: bool,
}

impl TransactionPreloadBuilder {
    pub fn block_hash(mut self, block_hash: String) -> Self {
        self.block_hash = block_hash;
        self
    }

    pub fn block_number(mut self, block_number: i64) -> Self {
        self.block_number = block_number;
        self
    }

    pub fn utxos(mut self, utxos: Vec<UTXO>) -> Self {
        self.utxos = utxos;
        self
    }

    pub fn sequence(mut self, sequence: u64) -> Self {
        self.sequence = sequence;
        self
    }

    pub fn chain_id(mut self, chain_id: String) -> Self {
        self.chain_id = chain_id;
        self
    }

    pub fn is_destination_address_exist(mut self, is_destination_address_exist: bool) -> Self {
        self.is_destination_address_exist = is_destination_address_exist;
        self
    }

    pub fn build(self) -> TransactionPreload {
        TransactionPreload {
            block_hash: self.block_hash,
            block_number: self.block_number,
            utxos: self.utxos,
            sequence: self.sequence,
            chain_id: self.chain_id,
            is_destination_address_exist: self.is_destination_address_exist,
        }
    }
}
