use std::{collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;
use primitives::{AssetIdVecExt, Transaction};
use storage::{models, DatabaseClient};
use streamer::{consumer::MessageConsumer, StreamProducer, TransactionsPayload};
use streamer::{AssetId, AssetsAddressPayload, NotificationsPayload, StreamProducerQueue};
use tokio::sync::Mutex;

use crate::consumers::TransactionsConsumerConfig;
use crate::pusher::Pusher;

pub struct TransactionsConsumer {
    pub database: Arc<Mutex<DatabaseClient>>,
    pub stream_producer: StreamProducer,
    pub pusher: Pusher,
    pub config: TransactionsConsumerConfig,
}

#[async_trait]
impl MessageConsumer<TransactionsPayload, usize> for TransactionsConsumer {
    async fn process(&mut self, payload: TransactionsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain = payload.chain;
        let transactions = payload
            .transactions
            .into_iter()
            .filter(|x| self.config.filter_transaction(x))
            .collect::<Vec<_>>();

        let addresses = transactions.clone().into_iter().flat_map(|x| x.addresses()).collect();
        let subscriptions = self.database.lock().await.get_subscriptions(chain, addresses)?;

        let mut transactions_map: HashMap<String, Transaction> = HashMap::new();
        let mut fetch_assets_payload: Vec<AssetId> = Vec::new();
        let mut notifications_payload: Vec<NotificationsPayload> = Vec::new();
        let mut address_assets_payload: Vec<AssetsAddressPayload> = Vec::new();

        for subscription in subscriptions {
            for transaction in transactions.clone() {
                if transaction.addresses().contains(&subscription.address) {
                    transactions_map.insert(transaction.clone().id, transaction.clone());

                    let device = self.database.lock().await.get_device_by_id(subscription.device_id)?;
                    let transaction = transaction.finalize(vec![subscription.address.clone()]).clone();

                    let assets_ids = transaction.asset_ids();
                    let (existing_assets, missing_assets_ids) = {
                        let existing_assets = self
                            .database
                            .lock()
                            .await
                            .get_assets(assets_ids.ids().clone())?
                            .into_iter()
                            .map(|x| x.as_primitive())
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
                        println!("outdated transaction: {}, created_at: {}", transaction.id, transaction.created_at);
                    } else if assets_ids.ids_set() == assets_ids.ids_set() {
                        if let Ok(notifications) = self
                            .pusher
                            .get_messages(device.as_primitive(), transaction.clone(), subscription.as_primitive(), existing_assets.clone())
                            .await
                        {
                            notifications_payload.push(NotificationsPayload::new(notifications));
                        }
                    }

                    let assets_addresses = transaction
                        .assets_addresses()
                        .into_iter()
                        .filter(|x| existing_assets.iter().any(|a| a.id == x.asset_id) && subscription.address == x.address)
                        .collect::<Vec<_>>();

                    if !assets_addresses.is_empty() {
                        address_assets_payload.push(AssetsAddressPayload::new(assets_addresses));
                    }
                }
            }
        }

        let _ = self.stream_producer.publish_fetch_assets(fetch_assets_payload).await;
        let _ = self.stream_producer.publish_notifications_transactions(notifications_payload).await;
        let _ = self.stream_producer.publish_store_assets_addresses_associations(address_assets_payload).await;

        Ok(self.store_batch(transactions_map.clone()).await?)
    }
}

impl TransactionsConsumer {
    async fn store_batch(&mut self, items: HashMap<String, Transaction>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut db_guard = self.database.lock().await;

        let primitive_transactions = items
            .into_values()
            .filter(|x| {
                if let Ok(assets) = db_guard.get_assets(x.asset_ids().ids().clone()) {
                    assets.into_iter().filter(|x| x.is_enabled).count() == x.asset_ids().len()
                } else {
                    false
                }
            })
            .collect::<Vec<Transaction>>();

        let transaction_chunks = primitive_transactions.chunks(300);

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

            db_guard.add_transactions(transactions_to_store.clone(), transaction_addresses_to_store.clone())?;
        }

        Ok(primitive_transactions.len())
    }
}
