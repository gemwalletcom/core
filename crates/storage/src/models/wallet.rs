use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::wallets)]
pub struct WalletRow {
    pub id: String,
    pub wallet_type: String,
}

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::wallets_subscriptions)]
pub struct WalletSubscriptionRow {
    pub wallet_id: String,
    pub device_id: i32,
    pub wallet_index: i32,
    pub chain: String,
    pub address: String,
}
