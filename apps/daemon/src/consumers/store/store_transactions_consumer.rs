use std::collections::HashSet;
use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use primitives::{AssetAddress, AssetIdVecExt, ConfigKey, Transaction, TransactionId, WalletId};
use storage::{AssetsAddressesRepository, AssetsRepository, ConfigCacher, Database, TransactionsRepository, WalletsRepository};
use streamer::{AssetId, DeviceStreamEvent, DeviceStreamPayload, NotificationsPayload, StreamProducer, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};

use crate::consumers::store::StoreTransactionsConsumerConfig;
use crate::pusher::Pusher;

const TRANSACTION_BATCH_SIZE: usize = 100;

pub struct StoreTransactionsConsumer {
    pub database: Database,
    pub config_cacher: ConfigCacher,
    pub stream_producer: StreamProducer,
    pub pusher: Pusher,
    pub config: StoreTransactionsConsumerConfig,
}

struct ProcessingResult {
    transactions: Vec<Transaction>,
    notifications: Vec<NotificationsPayload>,
    assets_addresses: Vec<AssetAddress>,
    device_events: Vec<DeviceStreamPayload>,
}

#[async_trait]
impl MessageConsumer<TransactionsPayload, usize> for StoreTransactionsConsumer {
    async fn should_process(&self, _payload: TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: TransactionsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain = payload.chain;
        let transactions = payload.transactions;
        let is_notify_devices = !payload.blocks.is_empty();

        let min_amount = self.config_cacher.get_f64(ConfigKey::TransactionsMinAmountUsd)?;

        let addresses: Vec<_> = transactions.iter().flat_map(|tx| tx.addresses()).collect::<HashSet<_>>().into_iter().collect();
        let subscriptions = self.database.wallets()?.get_subscriptions_by_chain_addresses(chain, addresses)?;

        let subscription_addresses: HashSet<_> = subscriptions.iter().map(|s| &s.address).collect();

        let asset_ids: Vec<AssetId> = transactions
            .iter()
            .filter(|x| x.addresses().iter().any(|addr| subscription_addresses.contains(addr)))
            .flat_map(|x| x.asset_ids())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let (existing_assets, missing_assets) = self.get_existing_and_missing_assets(asset_ids).await?;
        let existing_assets_map: HashMap<AssetId, primitives::AssetPriceMetadata> = existing_assets.into_iter().map(|asset| (asset.asset.asset.id.clone(), asset)).collect();

        let _ = self.stream_producer.publish_fetch_assets(missing_assets).await;

        let mut transactions_map: HashMap<TransactionId, Transaction> = HashMap::new();
        let mut notifications: Vec<NotificationsPayload> = Vec::new();
        let mut assets_addresses: HashSet<AssetAddress> = HashSet::new();
        let mut device_events_map: HashMap<(String, WalletId), (HashSet<TransactionId>, HashSet<AssetId>)> = HashMap::new();

        for subscription in &subscriptions {
            for transaction in &transactions {
                if !transaction.addresses().contains(&subscription.address) {
                    continue;
                }

                let transaction_asset_ids = transaction.asset_ids();

                if transaction_asset_ids.iter().any(|id| !existing_assets_map.contains_key(id)) {
                    continue;
                }

                let Some(asset_price) = existing_assets_map.get(&transaction.asset_id) else {
                    continue;
                };

                assets_addresses.extend(
                    transaction
                        .assets_addresses_with_fee()
                        .into_iter()
                        .filter(|x| existing_assets_map.contains_key(&x.asset_id) && subscription.address == x.address),
                );

                if self
                    .config
                    .is_transaction_insufficient_amount(transaction, &asset_price.asset.asset, asset_price.price, min_amount)
                {
                    continue;
                }

                transactions_map.entry(transaction.id.clone()).or_insert_with(|| transaction.clone());

                let key = (subscription.device.id.clone(), subscription.wallet_id.clone());
                let (txn_ids, asset_ids) = device_events_map.entry(key).or_default();
                txn_ids.insert(transaction.id.clone());
                asset_ids.extend(transaction_asset_ids.iter().cloned());

                let is_outdated = self.config.is_transaction_outdated(transaction.created_at.naive_utc(), chain);
                let should_notify = !is_outdated && is_notify_devices;

                if should_notify {
                    let assets: Vec<primitives::Asset> = transaction_asset_ids
                        .iter()
                        .filter_map(|id| existing_assets_map.get(id))
                        .map(|asset_price| asset_price.asset.asset.clone())
                        .collect();

                    if let Ok(push_notifications) = self.pusher.get_messages(subscription, transaction.clone(), assets).await {
                        notifications.push(NotificationsPayload::new(push_notifications));
                    }
                }
            }
        }

        let device_events = device_events_map
            .into_iter()
            .map(|((device_id, wallet_id), (transaction_ids, asset_ids))| DeviceStreamPayload {
                device_id,
                event: DeviceStreamEvent::Transactions {
                    wallet_id,
                    transaction_ids: transaction_ids.into_iter().collect(),
                    asset_ids: asset_ids.into_iter().collect(),
                },
            })
            .collect();

        let result = ProcessingResult {
            transactions: transactions_map.into_values().collect(),
            notifications,
            assets_addresses: assets_addresses.into_iter().collect(),
            device_events,
        };
        self.publish_results(result).await
    }
}

impl StoreTransactionsConsumer {
    async fn publish_results(&self, result: ProcessingResult) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions_count = self.store_transactions(result.transactions).await?;
        let _ = self.stream_producer.publish_notifications_transactions(result.notifications).await;

        if !result.assets_addresses.is_empty() {
            let _ = self.database.assets_addresses()?.add_assets_addresses(result.assets_addresses);
        }

        let _ = self.stream_producer.publish_device_stream_events(result.device_events).await;

        Ok(transactions_count)
    }

    async fn get_existing_and_missing_assets(&self, assets_ids: Vec<AssetId>) -> Result<(Vec<primitives::AssetPriceMetadata>, Vec<AssetId>), Box<dyn Error + Send + Sync>> {
        let assets_with_prices = self.database.assets()?.get_assets_with_prices(assets_ids.ids().clone())?;

        let missing_assets_ids = assets_ids
            .into_iter()
            .filter(|asset_id| !assets_with_prices.iter().any(|a| &a.asset.asset.id == asset_id))
            .collect::<Vec<_>>();

        Ok((assets_with_prices, missing_assets_ids))
    }

    async fn store_transactions(&self, transactions: Vec<Transaction>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if transactions.is_empty() {
            return Ok(0);
        }

        for chunk in transactions.chunks(TRANSACTION_BATCH_SIZE) {
            self.database.transactions()?.add_transactions(chunk.to_vec())?;
        }

        Ok(transactions.len())
    }
}
