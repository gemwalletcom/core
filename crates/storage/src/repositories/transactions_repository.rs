use std::error::Error;

use crate::DatabaseClient;
use crate::database::transactions::TransactionsStore;
use crate::models::Transaction;

pub trait TransactionsRepository {
    fn get_transactions_by_id(&mut self, _id: &str) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>>;
}

impl TransactionsRepository for DatabaseClient {
    fn get_transactions_by_id(&mut self, _id: &str) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(TransactionsStore::get_transactions_by_id(self, _id)?)
    }
}