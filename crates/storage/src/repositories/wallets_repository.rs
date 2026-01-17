use crate::database::wallets::WalletsStore;
use crate::models::{NewWalletRow, WalletRow, WalletSubscriptionRow};
use crate::{DatabaseClient, DatabaseError};
use primitives::Chain;
use std::collections::HashMap;

pub trait WalletsRepository {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRow, DatabaseError>;
    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRow, DatabaseError>;
    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRow>, DatabaseError>;
    fn create_wallets(&mut self, wallets: Vec<NewWalletRow>) -> Result<usize, DatabaseError>;
    fn get_or_create_wallet(&mut self, wallet: NewWalletRow) -> Result<WalletRow, DatabaseError>;
    fn get_subscriptions(&mut self, device_id: &str) -> Result<Vec<(WalletRow, WalletSubscriptionRow)>, DatabaseError>;
    fn add_subscriptions(&mut self, device_id: &str, wallet_ids: HashMap<String, i32>, subscriptions: Vec<(String, Vec<(Chain, String)>)>) -> Result<usize, DatabaseError>;
    fn delete_subscriptions(&mut self, device_id: &str, wallet_ids: HashMap<String, i32>, subscriptions: Vec<(String, Vec<(Chain, String)>)>) -> Result<usize, DatabaseError>;
}

impl WalletsRepository for DatabaseClient {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRow, DatabaseError> {
        WalletsStore::get_wallet(self, identifier)
    }

    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRow, DatabaseError> {
        WalletsStore::get_wallet_by_id(self, id)
    }

    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRow>, DatabaseError> {
        WalletsStore::get_wallets(self, identifiers)
    }

    fn create_wallets(&mut self, wallets: Vec<NewWalletRow>) -> Result<usize, DatabaseError> {
        WalletsStore::create_wallets(self, wallets)
    }

    fn get_or_create_wallet(&mut self, wallet: NewWalletRow) -> Result<WalletRow, DatabaseError> {
        match WalletsStore::get_wallet(self, &wallet.identifier) {
            Ok(existing) => Ok(existing),
            Err(DatabaseError::NotFound) => WalletsStore::create_wallet(self, wallet),
            Err(e) => Err(e),
        }
    }

    fn get_subscriptions(&mut self, device_id: &str) -> Result<Vec<(WalletRow, WalletSubscriptionRow)>, DatabaseError> {
        WalletsStore::get_subscriptions(self, device_id)
    }

    fn add_subscriptions(&mut self, device_id: &str, wallet_ids: HashMap<String, i32>, subscriptions: Vec<(String, Vec<(Chain, String)>)>) -> Result<usize, DatabaseError> {
        WalletsStore::add_subscriptions(self, device_id, wallet_ids, subscriptions)
    }

    fn delete_subscriptions(&mut self, device_id: &str, wallet_ids: HashMap<String, i32>, subscriptions: Vec<(String, Vec<(Chain, String)>)>) -> Result<usize, DatabaseError> {
        WalletsStore::delete_subscriptions(self, device_id, wallet_ids, subscriptions)
    }
}
