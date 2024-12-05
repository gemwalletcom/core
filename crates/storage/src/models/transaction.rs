use std::str::FromStr;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{transaction_utxo::TransactionInput, AssetId, TransactionDirection, TransactionType};
use serde::{Deserialize, Serialize};

// #[derive(FromSqlRow, Serialize, Deserialize, Debug, Default, AsExpression)]
// #[diesel(sql_type = Jsonb)]
// pub struct TransactionUTXO {
//pub address: String,
//pub value: String,
//}

//AsChangeset
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub id: String,
    pub chain: String,
    pub hash: String,
    pub memo: Option<String>,
    pub asset_id: String,
    pub value: Option<String>,
    pub fee: Option<String>,
    pub fee_asset_id: String,
    pub block_number: i32,
    pub sequence: Option<i32>,
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
        Self {
            id: transaction.id.to_string(),
            chain: transaction.asset_id.chain.as_ref().to_string(),
            hash: transaction.hash,
            memo: transaction.memo,
            asset_id: transaction.asset_id.to_string(),
            value: transaction.value.into(),
            fee: transaction.fee.into(),
            fee_asset_id: transaction.fee_asset_id.to_string(),
            block_number: transaction.block_number.parse::<i32>().unwrap_or_default(),
            sequence: transaction.sequence.parse::<i32>().unwrap_or_default().into(),
            from_address: transaction.from.into(),
            to_address: transaction.to.into(),
            kind: transaction.transaction_type.as_ref().to_string(),
            state: transaction.state.to_string(),
            created_at: transaction.created_at.naive_utc(),
            utxo_inputs: serde_json::to_value(transaction.utxo_inputs).ok(),
            utxo_outputs: serde_json::to_value(transaction.utxo_outputs).ok(),
            metadata: serde_json::to_value(transaction.metadata).ok(),
        }
    }

    pub fn as_primitive(&self, addresses: Vec<String>) -> primitives::Transaction {
        //TODO: Remove addresses from here
        let asset_id = AssetId::new(self.asset_id.clone().as_str()).unwrap();
        let hash = self.hash.clone();
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
        let transaction_type = TransactionType::from_str(self.kind.as_str()).ok().unwrap();

        primitives::Transaction::new_with_utxo(
            hash.clone(),
            asset_id,
            from.clone().into(),
            to_address.clone().into(),
            None,
            transaction_type,
            primitives::TransactionState::new(self.state.as_str()).unwrap(),
            self.block_number.to_string(),
            self.sequence.unwrap_or_default().to_string(),
            self.fee.clone().unwrap(),
            AssetId::new(self.fee_asset_id.clone().as_str()).unwrap(),
            self.value.clone().unwrap_or_default(),
            self.memo.clone(),
            direction,
            inputs.clone(),
            outputs.clone(),
            self.metadata.clone(),
            self.created_at.and_utc(),
        )
    }
}

pub struct TransactionC {
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub value: String,
    pub direction: TransactionDirection,
    pub inputs: Option<Vec<TransactionInput>>,
    pub outputs: Option<Vec<TransactionInput>>,
}
