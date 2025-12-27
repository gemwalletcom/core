use std::str::FromStr;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{AssetId, Chain, Transaction, TransactionDirection, TransactionId, TransactionState, TransactionType, TransactionUtxoInput};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionRow {
    pub id: i64,
    pub chain: String,
    pub hash: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub memo: Option<String>,
    pub state: String,
    pub kind: String,
    pub value: Option<String>,
    pub asset_id: String,
    pub fee: Option<String>,
    pub utxo_inputs: Option<serde_json::Value>,
    pub utxo_outputs: Option<serde_json::Value>,
    pub fee_asset_id: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionRow {
    pub chain: String,
    pub hash: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub memo: Option<String>,
    pub state: String,
    pub kind: String,
    pub value: Option<String>,
    pub asset_id: String,
    pub fee: Option<String>,
    pub utxo_inputs: Option<serde_json::Value>,
    pub utxo_outputs: Option<serde_json::Value>,
    pub fee_asset_id: String,
    pub metadata: Option<serde_json::Value>,
}

impl TransactionRow {
    pub fn get_addresses(&self) -> Vec<String> {
        vec![self.from_address.clone(), self.to_address.clone()].into_iter().flatten().collect()
    }

    pub fn as_primitive(&self, addresses: Vec<String>) -> Transaction {
        let chain = Chain::from_str(&self.chain).unwrap();
        let transaction_id = TransactionId::new(chain, self.hash.clone());
        let asset_id = AssetId::new(self.asset_id.clone().as_str()).unwrap();
        let from = self.from_address.clone().unwrap_or_default();
        let to_address = self.to_address.clone().unwrap_or_default();
        let inputs: Option<Vec<TransactionUtxoInput>> = serde_json::from_value(self.utxo_inputs.clone().into()).ok();
        let outputs: Option<Vec<TransactionUtxoInput>> = serde_json::from_value(self.utxo_outputs.clone().into()).ok();

        let direction = if addresses.contains(&from) {
            TransactionDirection::Outgoing
        } else if addresses.contains(&to_address) {
            TransactionDirection::Incoming
        } else {
            TransactionDirection::SelfTransfer
        };
        let transaction_type = TransactionType::from_str(self.kind.as_str()).ok().unwrap();

        Transaction {
            id: transaction_id.clone(),
            hash: self.hash.clone(),
            asset_id,
            from: from.clone(),
            to: to_address.clone(),
            contract: None,
            transaction_type,
            state: TransactionState::new(self.state.as_str()).unwrap(),
            block_number: None,
            sequence: None,
            fee: self.fee.clone().unwrap(),
            fee_asset_id: AssetId::new(self.fee_asset_id.clone().as_str()).unwrap(),
            value: self.value.clone().unwrap_or("0".to_string()),
            memo: self.memo.clone(),
            direction,
            utxo_inputs: inputs.unwrap_or_default().into(),
            utxo_outputs: outputs.unwrap_or_default().into(),
            metadata: self.metadata.clone(),
            created_at: self.created_at.and_utc(),
        }
    }
}

impl NewTransactionRow {
    pub fn from_primitive(transaction: Transaction) -> Self {
        let utxo_inputs = if transaction.utxo_inputs.clone().unwrap_or_default().is_empty() {
            None
        } else {
            serde_json::to_value(transaction.utxo_inputs.clone()).ok()
        };
        let utxo_outputs = if transaction.utxo_outputs.clone().unwrap_or_default().is_empty() {
            None
        } else {
            serde_json::to_value(transaction.clone().utxo_outputs.clone()).ok()
        };
        let metadata = if transaction.metadata.is_none() {
            None
        } else {
            serde_json::to_value(transaction.metadata.clone()).ok()
        };
        let from_address = if transaction.from.is_empty() { None } else { Some(transaction.from) };
        let to_address = if transaction.to.is_empty() { None } else { Some(transaction.to) };
        let value = if transaction.value.is_empty() || transaction.value == "0" {
            None
        } else {
            Some(transaction.value)
        };

        Self {
            chain: transaction.asset_id.chain.as_ref().to_string(),
            hash: transaction.hash,
            memo: transaction.memo,
            asset_id: transaction.asset_id.to_string(),
            value,
            fee: transaction.fee.into(),
            fee_asset_id: transaction.fee_asset_id.to_string(),
            from_address,
            to_address,
            kind: transaction.transaction_type.as_ref().to_string(),
            state: transaction.state.to_string(),
            utxo_inputs,
            utxo_outputs,
            metadata,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::transactions_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionTypeRow {
    pub id: String,
}

impl TransactionTypeRow {
    pub fn from_primitive(primitive: TransactionType) -> Self {
        Self {
            id: primitive.as_ref().to_owned(),
        }
    }
}
