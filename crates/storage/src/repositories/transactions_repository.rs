use crate::DatabaseClient;
use crate::DatabaseError;
use crate::database::transactions::TransactionsStore;
use crate::models::{AddressChainIdResultRow, TransactionRow};
use chrono::NaiveDateTime;
use primitives::Transaction;

pub trait TransactionsRepository {
    fn get_transaction_by_id(&mut self, chain: &str, hash: &str) -> Result<TransactionRow, DatabaseError>;
    fn add_transactions(&mut self, transactions: Vec<Transaction>) -> Result<usize, DatabaseError>;
    fn get_transactions_by_device_id(
        &mut self,
        _device_id: &str,
        addresses: Vec<String>,
        chains: Vec<String>,
        asset_id: Option<String>,
        from_datetime: Option<NaiveDateTime>,
    ) -> Result<Vec<TransactionRow>, DatabaseError>;
    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64) -> Result<Vec<AddressChainIdResultRow>, DatabaseError>;
    fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<usize, DatabaseError>;
    fn get_transactions_without_addresses(&mut self, limit: i64) -> Result<Vec<i64>, DatabaseError>;
    fn delete_transactions_by_ids(&mut self, ids: Vec<i64>) -> Result<usize, DatabaseError>;
}

impl TransactionsRepository for DatabaseClient {
    fn get_transaction_by_id(&mut self, chain: &str, hash: &str) -> Result<TransactionRow, DatabaseError> {
        Ok(TransactionsStore::get_transaction_by_id(self, chain, hash)?)
    }

    fn add_transactions(&mut self, transactions: Vec<Transaction>) -> Result<usize, DatabaseError> {
        Ok(TransactionsStore::add_transactions(self, transactions)?)
    }

    fn get_transactions_by_device_id(
        &mut self,
        _device_id: &str,
        addresses: Vec<String>,
        chains: Vec<String>,
        asset_id: Option<String>,
        from_datetime: Option<NaiveDateTime>,
    ) -> Result<Vec<TransactionRow>, DatabaseError> {
        Ok(TransactionsStore::get_transactions_by_device_id(self, _device_id, addresses, chains, asset_id, from_datetime)?)
    }

    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64) -> Result<Vec<AddressChainIdResultRow>, DatabaseError> {
        Ok(TransactionsStore::get_transactions_addresses(self, min_count, limit)?)
    }

    fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<usize, DatabaseError> {
        Ok(TransactionsStore::delete_transactions_addresses(self, addresses)?)
    }

    fn get_transactions_without_addresses(&mut self, limit: i64) -> Result<Vec<i64>, DatabaseError> {
        Ok(TransactionsStore::get_transactions_without_addresses(self, limit)?)
    }

    fn delete_transactions_by_ids(&mut self, ids: Vec<i64>) -> Result<usize, DatabaseError> {
        Ok(TransactionsStore::delete_transactions_by_ids(self, ids)?)
    }
}
