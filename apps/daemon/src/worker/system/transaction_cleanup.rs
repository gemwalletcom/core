use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::Duration;

use chrono::Utc;
use storage::models::SubscriptionAddressExcludeRow;
use storage::{Database, TransactionsRepository, WalletsRepository};

#[derive(Clone)]
pub struct TransactionCleanupConfig {
    pub address_max_count: i64,
    pub address_limit: usize,
    pub lookback: Duration,
}

#[derive(Clone)]
pub struct TransactionCleanup {
    database: Database,
    config: TransactionCleanupConfig,
}

impl TransactionCleanup {
    pub fn new(database: Database, config: TransactionCleanupConfig) -> Self {
        Self { database, config }
    }

    pub async fn cleanup(&self) -> Result<HashMap<String, usize>, Box<dyn Error + Send + Sync>> {
        let since = (Utc::now() - self.config.lookback).naive_utc();

        let heavy_addresses = self
            .database
            .transactions()?
            .get_transactions_addresses(self.config.address_max_count, self.config.address_limit as i64, since)?;

        if heavy_addresses.is_empty() {
            return Ok(HashMap::new());
        }

        let subscriptions_exclude: Vec<_> = heavy_addresses
            .iter()
            .map(|x| SubscriptionAddressExcludeRow {
                address: x.address.clone(),
                chain: x.chain_id.clone(),
            })
            .collect();
        self.database.wallets()?.add_subscriptions_exclude_addresses(subscriptions_exclude)?;

        let addresses: Vec<String> = heavy_addresses.into_iter().map(|x| x.address).collect();
        let total_addresses = addresses.len();

        let affected_transaction_ids = self.database.transactions()?.delete_transactions_addresses(addresses)?;
        let total_transactions_addresses = affected_transaction_ids.len();

        let unique_ids: Vec<i64> = affected_transaction_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();
        let total_deleted_transactions = self.database.transactions()?.delete_orphaned_transactions(unique_ids)?;

        Ok(HashMap::from([
            ("addresses".to_string(), total_addresses),
            ("transactions_addresses".to_string(), total_transactions_addresses),
            ("transactions_deleted".to_string(), total_deleted_transactions),
        ]))
    }
}
