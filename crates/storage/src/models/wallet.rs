use crate::sql_types::{ChainRow, WalletSource, WalletType};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::wallets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WalletRow {
    pub id: i32,
    pub identifier: String,
    pub wallet_type: WalletType,
    pub source: WalletSource,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::wallets)]
pub struct NewWalletRow {
    pub identifier: String,
    pub wallet_type: WalletType,
    pub source: WalletSource,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::wallets_subscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WalletSubscriptionRow {
    pub id: i32,
    pub wallet_id: i32,
    pub device_id: i32,
    pub chain: ChainRow,
    pub address: String,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::wallets_subscriptions)]
pub struct NewWalletSubscriptionRow {
    pub wallet_id: i32,
    pub device_id: i32,
    pub chain: ChainRow,
    pub address: String,
}

