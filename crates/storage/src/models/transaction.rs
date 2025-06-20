use std::str::FromStr;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{transaction_utxo::TransactionInput, AssetId, TransactionId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub id: String,
    pub chain: String,
    pub memo: Option<String>,
    pub asset_id: String,
    pub value: Option<String>,
    pub fee: Option<String>,
    pub fee_asset_id: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub kind: String,
    pub state: String,
    pub created_at: NaiveDateTime,
    pub utxo_inputs: Option<serde_json::Value>,
    pub utxo_outputs: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

impl Transaction {
    pub fn from_primitive(transaction: primitives::Transaction) -> Self {
        let utxo_inputs = if transaction.utxo_inputs.is_empty() {
            None
        } else {
            serde_json::to_value(transaction.utxo_inputs.clone()).ok()
        };
        let utxo_outputs = if transaction.clone().utxo_outputs.is_empty() {
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
            id: transaction.id,
            chain: transaction.asset_id.chain.as_ref().to_string(),
            memo: transaction.memo,
            asset_id: transaction.asset_id.to_string(),
            value,
            fee: transaction.fee.into(),
            fee_asset_id: transaction.fee_asset_id.to_string(),
            from_address,
            to_address,
            kind: transaction.transaction_type.as_ref().to_string(),
            state: transaction.state.to_string(),
            created_at: transaction.created_at.naive_utc(),
            utxo_inputs,
            utxo_outputs,
            metadata,
        }
    }

    pub fn as_primitive(&self, addresses: Vec<String>) -> primitives::Transaction {
        let transaction_id = TransactionId::from_str(&self.id.clone()).unwrap();
        let asset_id = AssetId::new(self.asset_id.clone().as_str()).unwrap();
        let hash = transaction_id.hash.clone();
        let from = self.from_address.clone().unwrap_or_default();
        let to_address = self.to_address.clone().unwrap_or_default();
        let inputs: Option<Vec<TransactionInput>> = serde_json::from_value(self.utxo_inputs.clone().into()).ok();
        let outputs: Option<Vec<TransactionInput>> = serde_json::from_value(self.utxo_outputs.clone().into()).ok();

        let direction = if addresses.contains(&from) {
            primitives::TransactionDirection::Outgoing
        } else if addresses.contains(&to_address) {
            primitives::TransactionDirection::Incoming
        } else {
            primitives::TransactionDirection::SelfTransfer
        };
        let transaction_type = primitives::TransactionType::from_str(self.kind.as_str()).ok().unwrap();

        primitives::Transaction::new_with_utxo(
            hash.clone(),
            asset_id,
            from.clone().into(),
            to_address.clone().into(),
            None,
            transaction_type,
            primitives::TransactionState::new(self.state.as_str()).unwrap(),
            self.fee.clone().unwrap(),
            AssetId::new(self.fee_asset_id.clone().as_str()).unwrap(),
            self.value.clone().unwrap_or("0".to_string()),
            self.memo.clone(),
            direction,
            inputs.clone(),
            outputs.clone(),
            self.metadata.clone(),
            self.created_at.and_utc(),
        )
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::transactions_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionType {
    pub id: String,
}

impl TransactionType {
    pub fn from_primitive(primitive: primitives::TransactionType) -> Self {
        Self {
            id: primitive.as_ref().to_owned(),
        }
    }
}
