use chrono::Utc;
use diesel::prelude::*;
use primitives::AssetId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub chain: String,
    pub hash: String,
    pub memo: Option<String>,
    pub asset_id: Option<String>,
    pub value: Option<String>,
    pub fee: Option<String>,
    pub fee_asset_id: Option<String>,
    pub block_number: i32,
    pub sequence: Option<i32>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub kind: String,
    pub state: String,
}

impl Transaction {
    pub fn from_primitive(transaction: primitives::Transaction) -> Self {
        Self{
            chain: transaction.asset_id.chain.as_str().to_string(),
            hash: transaction.hash,
            memo: transaction.memo,
            asset_id: transaction.asset_id.to_string().into(),
            value: transaction.value.into(),
            fee: transaction.fee.into(),
            fee_asset_id: transaction.fee_asset_id.to_string().into(),
            block_number: transaction.block_number.into(),
            sequence: transaction.sequence.into(),
            from_address: transaction.from.into(),
            to_address: transaction.to.into(),
            kind: transaction.transaction_type.to_string(),
            state: transaction.state.to_string(),
        }
    }

    pub fn as_primitive(&self, addresses: Vec<String>) -> primitives::Transaction {
        let from_address = self.from_address.clone().unwrap();
        let to_address = self.to_address.clone().unwrap();

        let direction = if addresses.contains(&from_address)  {
            primitives::TransactionDirection::Outgoing
        } else if addresses.contains(&to_address) {
            primitives::TransactionDirection::Incoming
        } else {
            primitives::TransactionDirection::SelfTransfer
        };

        let asset_id = AssetId::new(self.asset_id.clone().unwrap().as_str()).unwrap();
        let hash = self.hash.clone();

        return primitives::Transaction{
            id: primitives::Transaction::id_from(asset_id.chain, hash.clone()),
            hash: hash.clone(),
            asset_id: AssetId::new(self.asset_id.clone().unwrap().as_str()).unwrap(),
            from: from_address.clone(),
            to: to_address.clone(),
            contract: None,
            transaction_type: primitives::TransactionType::from_str(&self.kind.as_str()).unwrap_or_default(),
            state: primitives::TransactionState::from_str(&self.state.as_str()).unwrap(),
            block_number: self.block_number.into(),
            sequence: self.sequence.unwrap_or_default().into(),
            fee: self.fee.clone().unwrap(),
            fee_asset_id: AssetId::new(self.fee_asset_id.clone().unwrap().as_str()).unwrap(),
            value: self.value.clone().unwrap_or_default(),
            memo: self.memo.clone(),
            direction,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}