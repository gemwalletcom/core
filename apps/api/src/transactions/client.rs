use std::error::Error;

use chrono::{DateTime, Utc};
use primitives::{Transaction, TransactionId, TransactionsResponse};
use storage::{Database, ScanAddressesRepository, TransactionsRepository, WalletsRepository};

#[derive(Clone)]
pub struct TransactionsClient {
    database: Database,
}

impl TransactionsClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_transactions_by_wallet_id(
        &self,
        device_id: &str,
        device_row_id: i32,
        wallet_id: i32,
        asset_id: Option<String>,
        from_timestamp: Option<u64>,
    ) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.wallets()?.get_subscriptions_by_wallet_id(device_row_id, wallet_id)?;

        let addresses: Vec<String> = subscriptions.iter().map(|(_, addr)| addr.address.clone()).collect();
        let chains: Vec<String> = subscriptions.iter().map(|(sub, _)| sub.chain.0.as_ref().to_string()).collect();
        let from_datetime = from_timestamp.and_then(|ts| DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|dt| dt.naive_utc()));

        let transactions = self
            .database
            .transactions()?
            .get_transactions_by_device_id(device_id, addresses.clone(), chains, asset_id, from_datetime)?
            .into_iter()
            .map(|x| x.as_primitive(addresses.clone()).finalize(addresses.clone()))
            .collect::<Vec<Transaction>>();

        let scan_addresses = transactions.iter().flat_map(|x| x.addresses()).collect::<Vec<String>>();

        let address_names = self
            .database
            .scan_addresses()?
            .get_scan_addresses_by_addresses(scan_addresses)?
            .into_iter()
            .flat_map(|x| x.as_primitive())
            .collect();

        Ok(TransactionsResponse::new(transactions, address_names))
    }

    pub fn get_transaction_by_id(&self, id: &TransactionId) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.database.transactions()?.get_transaction_by_id(id.chain.as_ref(), &id.hash)?.as_primitive(vec![]))
    }
}
