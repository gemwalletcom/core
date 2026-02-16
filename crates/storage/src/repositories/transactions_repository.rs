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
    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64, since: NaiveDateTime) -> Result<Vec<AddressChainIdResultRow>, DatabaseError>;
    fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<i64>, DatabaseError>;
    fn delete_orphaned_transactions(&mut self, candidate_ids: Vec<i64>) -> Result<usize, DatabaseError>;
    fn get_asset_usage_counts(&mut self, since: NaiveDateTime) -> Result<Vec<(String, i64)>, DatabaseError>;
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
        Ok(TransactionsStore::get_transactions_by_device_id(
            self,
            _device_id,
            addresses,
            chains,
            asset_id,
            from_datetime,
        )?)
    }

    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64, since: NaiveDateTime) -> Result<Vec<AddressChainIdResultRow>, DatabaseError> {
        Ok(TransactionsStore::get_transactions_addresses(self, min_count, limit, since)?)
    }

    fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<i64>, DatabaseError> {
        Ok(TransactionsStore::delete_transactions_addresses(self, addresses)?)
    }

    fn delete_orphaned_transactions(&mut self, candidate_ids: Vec<i64>) -> Result<usize, DatabaseError> {
        Ok(TransactionsStore::delete_orphaned_transactions(self, candidate_ids)?)
    }

    fn get_asset_usage_counts(&mut self, since: NaiveDateTime) -> Result<Vec<(String, i64)>, DatabaseError> {
        Ok(TransactionsStore::get_asset_usage_counts(self, since)?)
    }
}
