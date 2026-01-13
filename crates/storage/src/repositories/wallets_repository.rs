use crate::database::wallets::WalletsStore;
use crate::models::{WalletRow, WalletSubscriptionRow};
use crate::{DatabaseClient, DatabaseError};

pub trait WalletsRepository {
    fn create_wallet(&mut self, wallet_id: &str, wallet_type: &str) -> Result<(), DatabaseError>;
    fn get_wallet(&mut self, wallet_id: &str) -> Result<Option<WalletRow>, DatabaseError>;
    fn add_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32, subscriptions: Vec<(String, String, i32)>) -> Result<usize, DatabaseError>;
    fn get_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32) -> Result<Vec<WalletSubscriptionRow>, DatabaseError>;
    fn delete_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32, subscriptions: Vec<(String, String, i32)>) -> Result<usize, DatabaseError>;
}

impl WalletsRepository for DatabaseClient {
    fn create_wallet(&mut self, wallet_id: &str, wallet_type: &str) -> Result<(), DatabaseError> {
        WalletsStore::create_wallet(self, wallet_id, wallet_type)
    }

    fn get_wallet(&mut self, wallet_id: &str) -> Result<Option<WalletRow>, DatabaseError> {
        WalletsStore::get_wallet(self, wallet_id)
    }

    fn add_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32, subscriptions: Vec<(String, String, i32)>) -> Result<usize, DatabaseError> {
        WalletsStore::add_wallet_subscriptions(self, wallet_id, device_id, subscriptions)
    }

    fn get_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32) -> Result<Vec<WalletSubscriptionRow>, DatabaseError> {
        WalletsStore::get_wallet_subscriptions(self, wallet_id, device_id)
    }

    fn delete_wallet_subscriptions(&mut self, wallet_id: &str, device_id: i32, subscriptions: Vec<(String, String, i32)>) -> Result<usize, DatabaseError> {
        WalletsStore::delete_wallet_subscriptions(self, wallet_id, device_id, subscriptions)
    }
}
