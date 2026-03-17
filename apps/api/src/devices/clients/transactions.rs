use std::error::Error;

use primitives::{AssetId, Chain, Transaction, TransactionId, TransactionsResponse};
use settings_chain::ChainProviders;
use storage::{Database, ScanAddressesRepository, TransactionsRepository, WalletsRepository};

use chrono::{DateTime, Utc};

const LIVE_SYNC_CHAINS: &[Chain] = &[Chain::HyperCore];

pub struct TransactionsClient {
    database: Database,
    providers: ChainProviders,
}

impl TransactionsClient {
    pub fn new(database: Database, providers: ChainProviders) -> Self {
        Self { database, providers }
    }

    async fn sync_transactions(&self, device_row_id: i32, wallet_id: i32, asset_id: &AssetId, from_timestamp: Option<u64>) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !LIVE_SYNC_CHAINS.contains(&asset_id.chain) {
            return Ok(());
        }
        let address = self.database.wallets()?.subscriptions_wallet_address_for_chain(device_row_id, wallet_id, asset_id.chain)?;
        let transactions = self
            .providers
            .get_transactions_by_address(asset_id.chain, address, from_timestamp)
            .await?
            .into_iter()
            .filter(|tx| tx.asset_id == *asset_id)
            .collect::<Vec<_>>();
        self.database.transactions()?.add_transactions(transactions)?;

        Ok(())
    }

    pub async fn get_transactions_by_wallet_id(
        &self,
        device_id: &str,
        device_row_id: i32,
        wallet_id: i32,
        asset_id: Option<AssetId>,
        from_timestamp: Option<u64>,
    ) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        if let Some(asset_id) = asset_id.as_ref() {
            let _ = self.sync_transactions(device_row_id, wallet_id, asset_id, from_timestamp).await;
        }
        let subscriptions = self.database.wallets()?.get_subscriptions_by_wallet_id(device_row_id, wallet_id)?;
        let addresses = subscriptions.iter().map(|(_, addr)| addr.address.clone()).collect::<Vec<_>>();
        let chains = subscriptions.iter().map(|(sub, _)| sub.chain.0.as_ref().to_string()).collect::<Vec<_>>();
        let from_datetime = from_timestamp.and_then(|ts| DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|dt| dt.naive_utc()));
        let transactions = self
            .database
            .transactions()?
            .get_transactions_by_device_id(device_id, addresses.clone(), chains, asset_id.map(|x| x.to_string()), from_datetime)?
            .into_iter()
            .map(|x| x.as_primitive(addresses.clone()).finalize(addresses.clone()))
            .collect::<Vec<_>>();
        let address_names = self
            .database
            .scan_addresses()?
            .get_scan_addresses_by_addresses(transactions.iter().flat_map(|x| x.addresses()).collect())?
            .into_iter()
            .filter_map(|x| x.as_primitive())
            .collect();

        Ok(TransactionsResponse::new(transactions, address_names))
    }

    pub fn get_transaction_by_id(&self, id: &TransactionId) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.database.transactions()?.get_transaction_by_id(id)?.as_primitive(vec![]))
    }
}
