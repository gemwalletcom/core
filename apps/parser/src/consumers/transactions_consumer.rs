use std::{collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;
use primitives::{AssetId, Transaction};
use storage::{models, DatabaseClient};
use streamer::{consumer::MessageConsumer, QueueName, StreamProducer, TransactionsPayload};
use streamer::{FetchAssetsPayload, NotificationsPayload};
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

        let subscriptions = {
            let mut db_guard = self.database.lock().await;
            db_guard.get_subscriptions(chain, addresses)?
        };

        let mut transactions_map: HashMap<String, Transaction> = HashMap::new();

        for subscription in subscriptions {
            for transaction in transactions.clone() {
                if transaction.addresses().contains(&subscription.address) {
                    let device = {
                        let mut db_guard = self.database.lock().await;
                        db_guard.get_device_by_id(subscription.device_id)?
                    };

                    println!(
                        "push: device: {}, chain: {}, transaction: {:?}",
                        subscription.device_id,
                        chain.as_ref(),
                        transaction.hash
                    );

                    transactions_map.insert(transaction.clone().id, transaction.clone());

                    let transaction_finalized = transaction.finalize(vec![subscription.address.clone()]).clone();

                    if self.config.is_transaction_outdated(transaction_finalized.created_at.naive_utc(), chain) {
                        println!(
                            "outdated transaction: {}, created_at: {}",
                            transaction_finalized.id, transaction_finalized.created_at
                        );
                        continue;
                    }

                    let assets_ids = transaction_finalized.asset_ids();
                    let (assets, missing_assets_ids) = {
                        let mut db_guard = self.database.lock().await;
                        let current_assets = db_guard
                            .get_assets(assets_ids.clone())?
                            .into_iter()
                            .map(|x| x.as_primitive())
                            .collect::<Vec<_>>();

                        let missing = assets_ids
                            .into_iter()
                            .filter(|asset_id_str| !current_assets.iter().any(|a| &a.id.to_string() == asset_id_str))
                            .filter_map(|x| AssetId::new(x.as_str()))
                            .collect::<Vec<_>>();
                        (current_assets, missing)
                    };

                    let notifications_result = self
                        .pusher
                        .get_messages(device.as_primitive(), transaction_finalized.clone(), subscription.as_primitive(), assets)
                        .await;

                    if let Ok(notifications) = notifications_result {
                        let _ = self
                            .stream_producer
                            .publish(QueueName::NotificationsTransactions, &NotificationsPayload::new(notifications))
                            .await;
                    }
                    if !missing_assets_ids.is_empty() {
                        let _ = self
                            .stream_producer
                            .publish_batch(
                                QueueName::FetchAssets,
                                &missing_assets_ids.into_iter().map(FetchAssetsPayload::new).collect::<Vec<_>>(),
                            )
                            .await;
                    }
                }
            }
        }

        match self.store_batch(transactions_map.clone()).await {
            Ok(_) => {}
            Err(err) => {
                println!("transaction insert: chain: {}, error: {:?}", chain.as_ref(), err);
            }
        }

        Ok(transactions_map.len())
    }
}

impl TransactionsConsumer {
    async fn store_batch(&mut self, items: HashMap<String, Transaction>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut db_guard = self.database.lock().await;

        let primitive_transactions = items
            .into_values()
            .filter(|x| {
                let asset_ids = x.asset_ids();
                if let Ok(assets) = db_guard.get_assets(asset_ids.clone()) {
                    assets.len() == asset_ids.len()
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
