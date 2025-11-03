use std::collections::HashSet;
use std::{collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;
use primitives::{AssetIdVecExt, Transaction, TransactionId};
use storage::{DatabaseClient, models};
use streamer::{AssetId, AssetsAddressPayload, NotificationsPayload, StreamProducer, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};
use tokio::sync::Mutex;

use crate::{consumers::StoreTransactionsConsumerConfig, pusher::Pusher};

pub struct StoreTransactionsConsumer {
    pub database: Arc<Mutex<DatabaseClient>>,
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
        let addresses = transactions.clone().into_iter().flat_map(|x| x.addresses()).collect();
        let subscriptions = self.database.lock().await.subscriptions().get_subscriptions(chain, addresses)?;

        let mut transactions_map: HashMap<TransactionId, Transaction> = HashMap::new();
        let mut fetch_assets_payload: Vec<AssetId> = Vec::new();
        let mut notifications_payload: Vec<NotificationsPayload> = Vec::new();
        let mut address_assets_payload: Vec<AssetsAddressPayload> = Vec::new();

        for subscription in subscriptions {
            for transaction in transactions.clone() {
                if transaction.addresses().contains(&subscription.subscription.address) {
                    let assets_ids = transaction.asset_ids();
                    let (existing_assets, missing_assets_ids) = self.get_existing_and_missing_assets(assets_ids.clone()).await?;
                    fetch_assets_payload.extend_from_slice(&missing_assets_ids);

                    if !missing_assets_ids.is_empty() {
                        continue;
                    }

                    transactions_map.insert(transaction.clone().id.clone(), transaction.clone());

                    let asset_price = existing_assets
                        .iter()
                        .find(|a| a.asset.asset.id == transaction.asset_id)
                        .cloned()
                        .expect("Asset must exist - already validated in missing_assets_ids check");

                    if !self
                        .config
                        .is_transaction_sufficient_amount(&transaction, &asset_price.asset.asset, asset_price.price, 0.01)
                    {
                        println!("insufficient amount, transaction: {}", transaction.id.clone(),);

                        transactions_map.remove(&transaction.id.clone());
                    } else if self.config.is_transaction_outdated(transaction.created_at.naive_utc(), chain) {
                        println!("outdated transaction: {}, created_at: {}", transaction.id.clone(), transaction.created_at);
                    } else if payload.blocks.is_empty() {
                        println!("empty blocks, transaction: {}, created_at: {}", transaction.id.clone(), transaction.created_at);
                    } else if assets_ids.ids_set() == assets_ids.ids_set() && is_notify_devices {
                        // important check is_notify_devices to avoid notifing users about transactions that are not parsed in the block
                        let assets: Vec<primitives::Asset> = existing_assets.iter().map(|x| x.asset.asset.clone()).collect();
                        if let Ok(notifications) = self
                            .pusher
                            .get_messages(
                                subscription.device.clone(),
                                transaction.clone(),
                                subscription.subscription.clone(),
                                assets.clone(),
                            )
                            .await
                        {
                            notifications_payload.push(NotificationsPayload::new(notifications));
                        }
                    }

                    let assets_addresses = transaction
                        .assets_addresses_with_fee()
                        .into_iter()
                        .filter(|x| existing_assets.iter().any(|a| a.asset.asset.id == x.asset_id) && subscription.subscription.address == x.address)
                        .collect::<Vec<_>>();

                    if !assets_addresses.is_empty() {
                        address_assets_payload.push(AssetsAddressPayload::new(assets_addresses.clone()));
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
        let assets_with_prices = self.database.lock().await.assets().get_assets_with_prices(assets_ids.ids().clone())?;

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
        let transaction_chunks = transactions.chunks(100);

        for chunk in transaction_chunks {
            let transactions_to_store = chunk
                .to_vec()
                .clone()
                .into_iter()
                .map(models::Transaction::from_primitive)
                .collect::<Vec<models::Transaction>>();

            let transaction_addresses_to_store = chunk
                .to_vec()
                .clone()
                .into_iter()
                .flat_map(models::TransactionAddresses::from_primitive)
                .collect::<HashSet<models::TransactionAddresses>>()
                .into_iter()
                .collect::<Vec<models::TransactionAddresses>>();

            if transactions_to_store.is_empty() || transaction_addresses_to_store.is_empty() {
                // If a chunk results in no data to store, skip to the next chunk.
                // The overall count of primitive_transactions will still be returned.
                continue;
            }

            self.database
                .lock()
                .await
                .transactions()
                .add_transactions(transactions_to_store.clone(), transaction_addresses_to_store.clone())?;
        }

        Ok(transactions.len())
    }
}
