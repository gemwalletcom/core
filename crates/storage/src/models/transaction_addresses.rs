use diesel::prelude::*;
use primitives::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::transactions_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionAddressesRow {
    pub id: i32,
    pub transaction_id: i64,
    pub asset_id: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, Clone, PartialEq, Eq, Hash)]
#[diesel(table_name = crate::schema::transactions_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionAddressesRow {
    pub transaction_id: i64,
    pub asset_id: String,
    pub address: String,
}

impl NewTransactionAddressesRow {
    pub fn from_transaction(transaction_id: i64, transaction: &Transaction) -> Vec<NewTransactionAddressesRow> {
        transaction
            .assets_addresses()
            .into_iter()
            .map(|x| Self {
                transaction_id,
                asset_id: x.asset_id.to_string(),
                address: x.address,
            })
            .collect()
    }
}

#[derive(Queryable, Debug, Clone)]
pub struct AddressChainIdResultRow {
    pub address: String,
    pub chain_id: String,
}
