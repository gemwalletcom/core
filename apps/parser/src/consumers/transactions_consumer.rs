use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use primitives::{AssetId, Transaction};
use storage::{models, DatabaseClient};
use streamer::{consumer::MessageConsumer, QueueName, StreamProducer, TransactionsPayload};
use streamer::{FetchAssetsPayload, NotificationsPayload};

use crate::consumers::TransactionsConsumerConfig;
use crate::pusher::Pusher;
pub struct TransactionsConsumer {
    pub database: DatabaseClient,
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
        let subscriptions = self.database.get_subscriptions(chain, addresses)?;
        let mut transactions_map: HashMap<String, Transaction> = HashMap::new();

        // Debugging only, insert all transactions
        // for transaction in transactions.clone().into_iter() {
        //     transactions_map.insert(transaction.clone().id, transaction.clone());
        // }

        for subscription in subscriptions {
            for transaction in transactions.clone() {
                if transaction.addresses().contains(&subscription.address) {
                    let device = self.database.get_device_by_id(subscription.device_id)?;

                    println!(
                        "push: device: {}, chain: {}, transaction: {:?}",
                        subscription.device_id,
                        chain.as_ref(),
                        transaction.hash
                    );

                    transactions_map.insert(transaction.clone().id, transaction.clone());

                    let transaction = transaction.finalize(vec![subscription.address.clone()]).clone();

                    if self.config.is_transaction_outdated(transaction.created_at.naive_utc(), chain) {
                        println!("outdated transaction: {}, created_at: {}", transaction.id, transaction.created_at);
                        continue;
                    }

                    let assets_ids = transaction.asset_ids();
                    let assets = self
                        .database
                        .get_assets(assets_ids.clone())?
                        .into_iter()
                        .map(|x| x.as_primitive())
                        .collect::<Vec<_>>();

                    let missing_assets_ids = assets_ids
                        .into_iter()
                        .filter(|asset_id| !assets.iter().any(|a| &a.id.to_string() == asset_id))
                        .filter_map(|x| AssetId::new(x.as_str()))
                        .collect::<Vec<_>>();

                    let notifications_result = self
                        .pusher
                        .get_messages(device.as_primitive(), transaction.clone(), subscription.as_primitive(), assets)
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
                            .publish(QueueName::FetchAssets, &FetchAssetsPayload::new(missing_assets_ids))
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
        let primitive_transactions = items
            .into_values()
            .filter(|x| {
                let asset_ids = x.asset_ids();
                if let Ok(assets) = self.database.get_assets(asset_ids.clone()) {
                    assets.len() == asset_ids.len()
                } else {
                    false
                }
            })
            .collect::<Vec<Transaction>>();

        let transaction_chunks = primitive_transactions.chunks(300);

        for chunk in transaction_chunks {
            let transactions = chunk
                .to_vec()
                .clone()
                .into_iter()
                .map(models::Transaction::from_primitive)
                .collect::<Vec<models::Transaction>>();

            let transaction_addresses = chunk
                .to_vec()
                .clone()
                .into_iter()
                .flat_map(models::TransactionAddresses::from_primitive)
                .collect::<Vec<models::TransactionAddresses>>();

            if transactions.is_empty() || transaction_addresses.is_empty() {
                return Ok(primitive_transactions.len());
            }

            self.database.add_transactions(transactions.clone(), transaction_addresses.clone())?;
        }

        Ok(primitive_transactions.len())
    }
}
