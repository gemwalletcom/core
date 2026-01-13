use crate::models::{WalletRow, WalletSubscriptionRow};
use crate::schema::{wallets, wallets_subscriptions};
use crate::{DatabaseClient, DatabaseError};
use diesel::prelude::*;

pub trait WalletsStore {
    fn create_wallet(&mut self, wallet_id: &str, wallet_type: &str) -> Result<(), DatabaseError>;
    fn get_wallet(&mut self, wallet_id: &str) -> Result<Option<WalletRow>, DatabaseError>;
    fn add_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32, subscriptions: Vec<(String, String, i32)>) -> Result<usize, DatabaseError>;
    fn get_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32) -> Result<Vec<WalletSubscriptionRow>, DatabaseError>;
    fn delete_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32, subscriptions: Vec<(String, String, i32)>) -> Result<usize, DatabaseError>;
}

impl WalletsStore for DatabaseClient {
    fn create_wallet(&mut self, wallet_id: &str, wallet_type: &str) -> Result<(), DatabaseError> {
        let new_wallet = WalletRow {
            id: wallet_id.to_string(),
            wallet_type: wallet_type.to_string(),
        };

        diesel::insert_into(wallets::table)
            .values(&new_wallet)
            .on_conflict(wallets::id)
            .do_nothing()
            .execute(&mut self.connection)?;

        Ok(())
    }

    fn get_wallet(&mut self, wallet_id: &str) -> Result<Option<WalletRow>, DatabaseError> {
        let wallet = wallets::table
            .filter(wallets::id.eq(wallet_id))
            .first::<WalletRow>(&mut self.connection)
            .optional()?;

        Ok(wallet)
    }

    fn add_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32, subscriptions: Vec<(String, String, i32)>) -> Result<usize, DatabaseError> {
        let rows: Vec<WalletSubscriptionRow> = subscriptions
            .into_iter()
            .map(|(chain, address, wallet_index)| WalletSubscriptionRow {
                wallet_id: wallet_id.to_string(),
                device_id,
                wallet_index,
                chain,
                address,
            })
            .collect();

        let count = diesel::insert_into(wallets_subscriptions::table)
            .values(&rows)
            .on_conflict((wallets_subscriptions::wallet_id, wallets_subscriptions::device_id, wallets_subscriptions::wallet_index, wallets_subscriptions::chain, wallets_subscriptions::address))
            .do_nothing()
            .execute(&mut self.connection)?;

        Ok(count)
    }

    fn get_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32) -> Result<Vec<WalletSubscriptionRow>, DatabaseError> {
        let subscriptions = wallets_subscriptions::table
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .load::<WalletSubscriptionRow>(&mut self.connection)?;

        Ok(subscriptions)
    }

    fn delete_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32, subscriptions: Vec<(String, String, i32)>) -> Result<usize, DatabaseError> {
        let mut count = 0;
        for (chain, address, wallet_index) in subscriptions {
            let deleted = diesel::delete(wallets_subscriptions::table)
                .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
                .filter(wallets_subscriptions::device_id.eq(device_id))
                .filter(wallets_subscriptions::wallet_index.eq(wallet_index))
                .filter(wallets_subscriptions::chain.eq(&chain))
                .filter(wallets_subscriptions::address.eq(&address))
                .execute(&mut self.connection)?;
            count += deleted;
        }

        Ok(count)
    }
}
