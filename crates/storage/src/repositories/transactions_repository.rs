use crate::database::transactions::{TransactionFilter, TransactionUpdate, TransactionsStore};
use crate::models::{AddressChainIdResultRow, TransactionRow};
use crate::sql_types::TransactionType;
use crate::{DatabaseClient, DatabaseError, DieselResultExt};
use chrono::NaiveDateTime;
use primitives::{AssetId, Transaction, TransactionId};

pub trait TransactionsRepository {
    fn get_transaction_by_id(&mut self, id: &TransactionId) -> Result<TransactionRow, DatabaseError>;
    fn get_transaction_exists(&mut self, id: &TransactionId) -> Result<bool, DatabaseError>;
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
    fn get_asset_usage_counts(&mut self, since: NaiveDateTime) -> Result<Vec<(AssetId, i64)>, DatabaseError>;
    fn get_transactions_by_filter(&mut self, filters: Vec<TransactionFilter>, limit: i64) -> Result<Vec<TransactionRow>, DatabaseError>;
    fn update_transaction(&mut self, chain: &str, hash: &str, updates: Vec<TransactionUpdate>) -> Result<usize, DatabaseError>;
    fn get_addresses_by_chain_and_kind(&mut self, chain: &str, kinds: Vec<TransactionType>, since: NaiveDateTime) -> Result<Vec<String>, DatabaseError>;
}

impl TransactionsRepository for DatabaseClient {
    fn get_transaction_by_id(&mut self, id: &TransactionId) -> Result<TransactionRow, DatabaseError> {
        TransactionsStore::get_transaction_by_id(self, id.chain.as_ref(), &id.hash).or_not_found(id.to_string())
    }

    fn get_transaction_exists(&mut self, id: &TransactionId) -> Result<bool, DatabaseError> {
        Ok(TransactionsStore::get_transaction_exists(self, id.chain.as_ref(), &id.hash)?)
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

    fn get_asset_usage_counts(&mut self, since: NaiveDateTime) -> Result<Vec<(AssetId, i64)>, DatabaseError> {
        Ok(TransactionsStore::get_asset_usage_counts(self, since)?
            .into_iter()
            .map(|(asset_id, count)| (asset_id.into(), count))
            .collect())
    }

    fn get_transactions_by_filter(&mut self, filters: Vec<TransactionFilter>, limit: i64) -> Result<Vec<TransactionRow>, DatabaseError> {
        Ok(TransactionsStore::get_transactions_by_filter(self, filters, limit)?)
    }

    fn update_transaction(&mut self, chain: &str, hash: &str, updates: Vec<TransactionUpdate>) -> Result<usize, DatabaseError> {
        Ok(TransactionsStore::update_transaction(self, chain, hash, updates)?)
    }

    fn get_addresses_by_chain_and_kind(&mut self, chain: &str, kinds: Vec<TransactionType>, since: NaiveDateTime) -> Result<Vec<String>, DatabaseError> {
        Ok(TransactionsStore::get_addresses_by_chain_and_kind(self, chain, kinds, since)?)
    }
}
