use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transactions_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionAssets {
    pub transaction_id: String,
    pub asset_id: String,
}
