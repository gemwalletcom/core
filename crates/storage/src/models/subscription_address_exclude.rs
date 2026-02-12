use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::subscriptions_addresses_exclude)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SubscriptionAddressExcludeRow {
    pub address: String,
    pub chain: String,
}
