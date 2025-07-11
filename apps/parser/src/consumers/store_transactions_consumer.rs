use std::collections::HashSet;
use std::{collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;
use primitives::{AssetIdVecExt, Transaction};
use storage::{models, DatabaseClient};
use streamer::{consumer::MessageConsumer, StreamProducer, TransactionsPayload};
use streamer::{AssetId, AssetsAddressPayload, NotificationsPayload, StreamProducerQueue};
use tokio::sync::Mutex;

use crate::consumers::StoreTransactionsConsumerConfig;
use crate::pusher::Pusher;

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
        let transactions = payload
            .transactions
            .into_iter()
            .filter(|x| self.config.filter_transaction(x))
            .collect::<Vec<_>>();
        let is_notify_devices = !payload.blocks.is_empty();
        let addresses = transactions.clone().into_iter().flat_map(|x| x.addresses()).collect();
        let subscriptions = self.database.lock().await.subscriptions().get_subscriptions(chain, addresses)?;

        let mut transactions_map: HashMap<String, Transaction> = HashMap::new();
        let mut fetch_assets_payload: Vec<AssetId> = Vec::new();
        let mut notifications_payload: Vec<NotificationsPayload> = Vec::new();
        let mut address_assets_payload: Vec<AssetsAddressPayload> = Vec::new();

        for subscription in subscriptions {
            for transaction in transactions.clone() {
                if transaction.addresses().contains(&subscription.subscription.address) {
                    transactions_map.insert(transaction.clone().id.clone(), transaction.clone());

                    let transaction = transaction.finalize(vec![subscription.subscription.address.clone()]).clone();

                    let assets_ids = transaction.asset_ids();
                    let (existing_assets, missing_assets_ids) = {
                        let existing_assets = self
                            .database
                            .lock()
                            .await
                            .assets()
                            .get_assets(assets_ids.ids().clone())?
                            .into_iter()
                            .collect::<Vec<_>>();

                        let missing_assets_ids = assets_ids
                            .clone()
                            .into_iter()
                            .filter(|asset_id| !existing_assets.iter().any(|a| &a.id == asset_id))
                            .collect::<Vec<_>>();
                        (existing_assets, missing_assets_ids)
                    };

                    fetch_assets_payload.extend_from_slice(&missing_assets_ids);

                    if self.config.is_transaction_outdated(transaction.created_at.naive_utc(), chain) {
                        println!("outdated transaction: {}, created_at: {}", transaction.id.clone(), transaction.created_at);
                    } else if payload.blocks.is_empty() {
                        println!("empty blocks, transaction: {}, created_at: {}", transaction.id.clone(), transaction.created_at);
                    } else if assets_ids.ids_set() == assets_ids.ids_set() && is_notify_devices {
                        // important check is_notify_devices to avoid notifing users about transactions that are not parsed in the block
                        if let Ok(notifications) = self
                            .pusher
                            .get_messages(
                                subscription.device.clone(),
                                transaction.clone(),
                                subscription.subscription.clone(),
                                existing_assets.clone(),
                            )
                            .await
                        {
                            notifications_payload.push(NotificationsPayload::new(notifications));
                        }
                    }

                    let assets_addresses = transaction
                        .assets_addresses()
                        .into_iter()
                        .filter(|x| existing_assets.iter().any(|a| a.id == x.asset_id) && subscription.subscription.address == x.address)
                        .collect::<Vec<_>>();

                    if !assets_addresses.is_empty() {
                        address_assets_payload.push(AssetsAddressPayload::new(assets_addresses.clone()));
                    }
                }
            }
        }

        let transactions_count = self.store_transactions(transactions_map.clone()).await?;
        let _ = self.stream_producer.publish_fetch_assets(fetch_assets_payload).await;
        let _ = self.stream_producer.publish_notifications_transactions(notifications_payload).await;
        let _ = self.stream_producer.publish_store_assets_addresses_associations(address_assets_payload).await;
        Ok(transactions_count)
    }
}

impl StoreTransactionsConsumer {
    async fn store_transactions(&mut self, transactions: HashMap<String, Transaction>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions_asset_ids: Vec<String> = transactions.values().flat_map(|x| x.asset_ids().ids()).collect();
        let enabled_asset_ids: HashSet<String> = self
            .database
            .lock()
            .await
            .assets()
            .get_assets_basic(transactions_asset_ids)?
            .into_iter()
            .filter(|x| x.properties.is_enabled)
            .map(|x| x.asset.id.to_string())
            .collect();

        let transactions = transactions
            .into_values()
            .filter(|x| x.asset_ids().ids().iter().all(|asset_id| enabled_asset_ids.contains(asset_id)))
            .collect::<Vec<Transaction>>();

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
