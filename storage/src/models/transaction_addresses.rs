use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transactions_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionAddresses {
    pub transaction_id: String,
    pub address: String,
}