use diesel::prelude::*;
use primitives::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone, PartialEq, Eq, Hash)]
#[diesel(table_name = crate::schema::transactions_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionAddressesRow {
    pub chain_id: String,
    pub asset_id: String,
    pub transaction_id: String,
    pub address: String,
}

impl TransactionAddressesRow {
    pub fn from_primitive(transaction: Transaction) -> Vec<TransactionAddressesRow> {
        let transaction_id = transaction.id.clone();
        transaction
            .assets_addresses()
            .into_iter()
            .map(|x| Self {
                chain_id: x.asset_id.chain.as_ref().to_string(),
                asset_id: x.asset_id.to_string(),
                transaction_id: transaction_id.to_string(),
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
