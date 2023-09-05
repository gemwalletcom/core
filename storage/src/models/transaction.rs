use diesel::prelude::*;
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
            kind: primitives::TransactionType::Transfer.to_string(),
        }
    }
}