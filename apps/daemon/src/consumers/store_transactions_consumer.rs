use std::collections::HashSet;
use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use primitives::{AssetIdVecExt, Transaction, TransactionId};
use storage::Database;
use storage::models;
use streamer::{AssetId, AssetsAddressPayload, NotificationsPayload, StreamProducer, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};

use crate::{consumers::StoreTransactionsConsumerConfig, pusher::Pusher};

const MIN_TRANSACTION_AMOUNT_USD: f64 = 0.01;
const TRANSACTION_BATCH_SIZE: usize = 100;

pub struct StoreTransactionsConsumer {
    pub database: Database,
    pub stream_producer: StreamProducer,
    pub pusher: Pusher,
    pub config: StoreTransactionsConsumerConfig,
}

#[async_trait]
impl MessageConsumer<TransactionsPayload, usize> for StoreTransactionsConsumer {
    async fn should_process(&mut self, _payload: TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }
    async fn process(&mut self, payload: TransactionsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain = payload.chain;
        let transactions = payload.transactions;
        let is_notify_devices = !payload.blocks.is_empty();

        let addresses: Vec<_> = transactions.iter().flat_map(|tx| tx.addresses()).collect::<HashSet<_>>().into_iter().collect();
        let subscriptions = self.database.client()?.subscriptions().get_subscriptions(chain, addresses)?;

        let subscription_addresses: HashSet<_> = subscriptions.iter().map(|s| &s.subscription.address).collect();

        let asset_ids: Vec<AssetId> = transactions
            .iter()
            .filter(|x| x.addresses().iter().any(|addr| subscription_addresses.contains(addr)))
            .flat_map(|x| x.asset_ids())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let (existing_assets, missing_assets_ids) = self.get_existing_and_missing_assets(asset_ids).await?;
        let existing_assets_map: HashMap<AssetId, primitives::AssetPriceMetadata> =
            existing_assets.into_iter().map(|asset| (asset.asset.asset.id.clone(), asset)).collect();

        let mut transactions_map: HashMap<TransactionId, Transaction> = HashMap::new();
        let mut fetch_assets_payload: Vec<AssetId> = Vec::new();
        let mut notifications_payload: Vec<NotificationsPayload> = Vec::new();
        let mut address_assets_payload: Vec<AssetsAddressPayload> = Vec::new();

        if !missing_assets_ids.is_empty() {
            fetch_assets_payload.extend(missing_assets_ids);
        }

        for subscription in &subscriptions {
            for transaction in &transactions {
                if !transaction.addresses().contains(&subscription.subscription.address) {
                    continue;
                }

                let transaction_asset_ids = transaction.asset_ids();

                if transaction_asset_ids.iter().any(|id| !existing_assets_map.contains_key(id)) {
                    continue;
                }

                let Some(asset_price) = existing_assets_map.get(&transaction.asset_id) else {
                    continue;
                };

                let assets_addresses = transaction
                    .assets_addresses_with_fee()
                    .into_iter()
                    .filter(|x| existing_assets_map.contains_key(&x.asset_id) && subscription.subscription.address == x.address)
                    .collect::<Vec<_>>();

                address_assets_payload.push(AssetsAddressPayload::new(assets_addresses));

                if self
                    .config
                    .is_transaction_insufficient_amount(transaction, &asset_price.asset.asset, asset_price.price, MIN_TRANSACTION_AMOUNT_USD)
                {
                    continue;
                }

                transactions_map.insert(transaction.id.clone(), transaction.clone());

                let is_outdated = self.config.is_transaction_outdated(transaction.created_at.naive_utc(), chain);
                let should_notify = !is_outdated && is_notify_devices;

                if should_notify {
                    let assets: Vec<primitives::Asset> = transaction_asset_ids
                        .iter()
                        .filter_map(|id| existing_assets_map.get(id))
                        .map(|asset_price| asset_price.asset.asset.clone())
                        .collect();

                    if let Ok(notifications) = self
                        .pusher
                        .get_messages(subscription.device.clone(), transaction.clone(), subscription.subscription.clone(), assets)
                        .await
                    {
                        notifications_payload.push(NotificationsPayload::new(notifications));
                    }
                }
            }
        }

        let transactions_count = self.store_transactions(transactions_map.into_values().collect()).await?;
        let _ = self.stream_producer.publish_fetch_assets(fetch_assets_payload).await;
        let _ = self.stream_producer.publish_notifications_transactions(notifications_payload).await;
        let _ = self.stream_producer.publish_store_assets_addresses_associations(address_assets_payload).await;
        Ok(transactions_count)
    }
}

impl StoreTransactionsConsumer {
    async fn get_existing_and_missing_assets(
        &mut self,
        assets_ids: Vec<AssetId>,
    ) -> Result<(Vec<primitives::AssetPriceMetadata>, Vec<AssetId>), Box<dyn Error + Send + Sync>> {
        let assets_with_prices = self.database.client()?.assets().get_assets_with_prices(assets_ids.ids().clone())?;

        let missing_assets_ids = assets_ids
            .into_iter()
            .filter(|asset_id| !assets_with_prices.iter().any(|a| &a.asset.asset.id == asset_id))
            .collect::<Vec<_>>();

        Ok((assets_with_prices, missing_assets_ids))
    }

    async fn store_transactions(&mut self, transactions: Vec<Transaction>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if transactions.is_empty() {
            return Ok(0);
        }

        for chunk in transactions.chunks(TRANSACTION_BATCH_SIZE) {
            let transactions_to_store: Vec<models::Transaction> = chunk.iter().map(|tx| models::Transaction::from_primitive(tx.clone())).collect();

            let transaction_addresses_to_store: Vec<models::TransactionAddresses> = chunk
                .iter()
                .flat_map(|tx| models::TransactionAddresses::from_primitive(tx.clone()))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            if transactions_to_store.is_empty() || transaction_addresses_to_store.is_empty() {
                continue;
            }

            self.database
                .client()?
                .transactions()
                .add_transactions(transactions_to_store, transaction_addresses_to_store)?;
        }

        Ok(transactions.len())
    }
}
