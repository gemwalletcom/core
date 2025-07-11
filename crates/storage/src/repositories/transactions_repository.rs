use std::error::Error;

use crate::DatabaseClient;
use crate::database::transactions::TransactionsStore;
use crate::models::{Transaction, TransactionAddresses, AddressChainIdResult, TransactionType};
use primitives::TransactionsFetchOption;

pub trait TransactionsRepository {
    fn get_transactions_by_id(&mut self, _id: &str) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>>;
    fn add_transactions(&mut self, transactions_values: Vec<Transaction>, addresses_values: Vec<TransactionAddresses>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    fn get_transactions_by_device_id(&mut self, _device_id: &str, addresses: Vec<String>, chains: Vec<String>, options: TransactionsFetchOption) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>>;
    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64) -> Result<Vec<AddressChainIdResult>, Box<dyn Error + Send + Sync>>;
    fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_transactions_without_addresses(&mut self, limit: i64) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
    fn delete_transactions_by_ids(&mut self, ids: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn add_transactions_types(&mut self, values: Vec<TransactionType>) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl TransactionsRepository for DatabaseClient {
    fn get_transactions_by_id(&mut self, _id: &str) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(TransactionsStore::get_transactions_by_id(self, _id)?)
    }

    fn add_transactions(&mut self, transactions_values: Vec<Transaction>, addresses_values: Vec<TransactionAddresses>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(TransactionsStore::add_transactions(self, transactions_values, addresses_values)?)
    }

    fn get_transactions_by_device_id(&mut self, _device_id: &str, addresses: Vec<String>, chains: Vec<String>, options: TransactionsFetchOption) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(TransactionsStore::get_transactions_by_device_id(self, _device_id, addresses, chains, options)?)
    }

    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64) -> Result<Vec<AddressChainIdResult>, Box<dyn Error + Send + Sync>> {
        Ok(TransactionsStore::get_transactions_addresses(self, min_count, limit)?)
    }

    fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(TransactionsStore::delete_transactions_addresses(self, addresses)?)
    }

    fn get_transactions_without_addresses(&mut self, limit: i64) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(TransactionsStore::get_transactions_without_addresses(self, limit)?)
    }

    fn delete_transactions_by_ids(&mut self, ids: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(TransactionsStore::delete_transactions_by_ids(self, ids)?)
    }

    fn add_transactions_types(&mut self, values: Vec<TransactionType>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(TransactionsStore::add_transactions_types(self, values)?)
    }
}