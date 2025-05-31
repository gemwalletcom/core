use std::{collections::HashMap, error::Error};

use api_connector::PusherClient;
use primitives::Transaction;
use settings::Settings;
use std::time::Instant;
use storage::DatabaseClient;
use streamer::{QueueName, StreamReader, TransactionsPayload};

use crate::{transactions_consumer_config::TransactionsConsumerConfig, Pusher};

pub async fn run_consumer_mode() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let settings = Settings::new()?;

    println!("parser consumer init");

    let mut reader = StreamReader::new(&settings.rabbitmq.url)
        .await
        .map_err(|e| format!("Failed to create consumer: {}", e))?;

    println!("parser consumer start");

    let pusher_client = PusherClient::new(settings.pusher.url.clone(), settings.pusher.ios.topic.clone());
    let pusher = Pusher::new(settings.postgres.url.clone(), pusher_client);
    let database_client = DatabaseClient::new(settings.postgres.url.as_str());

    let mut consumer = TransactionsConsumer {
        database: database_client,
        pusher,
        config: TransactionsConsumerConfig::default(),
    };

    reader
        .read::<TransactionsPayload, _>(QueueName::Transactions, |payload| {
            println!(
                "parser consumer received message: chain: {}, blocks: {:?}, transactions: {},",
                payload.chain,
                payload.blocks,
                payload.transactions.len()
            );

            let start = Instant::now();
            let result = tokio::task::block_in_place(|| {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async { consumer.process(payload.clone()).await })
            });
            let elapsed = start.elapsed();

            match &result {
                Ok(size) => {
                    println!(
                        "parser consumer processed: chain: {}, blocks: {:?}, insert transactions: {}, elapsed: {:?}",
                        payload.chain,
                        payload.blocks.clone(),
                        size,
                        elapsed
                    );
                }
                Err(error) => {
                    println!(
                        "parser consumer failed: chain: {}, blocks: {:?}, transactions: {}, elapsed: {:?}, error: {}",
                        payload.chain,
                        payload.blocks.clone(),
                        payload.transactions.clone().len(),
                        elapsed,
                        error
                    );
                }
            }

            result.map(|_| ())
        })
        .await?;

    Ok(())
}

pub struct TransactionsConsumer {
    pub database: DatabaseClient,
    pub pusher: Pusher,
    pub config: TransactionsConsumerConfig,
}

impl TransactionsConsumer {
    pub async fn process(&mut self, payload: TransactionsPayload) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
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

                    match self.pusher.push(device.as_primitive(), transaction, subscription.as_primitive()).await {
                        Ok(result) => {
                            println!("push: result: {:?}", result);
                        }
                        Err(err) => {
                            println!("push: error: {:?}", err);
                        }
                    }
                }
            }
        }

        match self.store_transactions(transactions_map.clone()).await {
            Ok(_) => {}
            Err(err) => {
                println!("transaction insert: chain: {}, error: {:?}", chain.as_ref(), err);
            }
        }

        Ok(transactions_map.len())
    }

    pub async fn store_transactions(&mut self, transactions_map: HashMap<String, primitives::Transaction>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let primitive_transactions = transactions_map
            .clone()
            .into_iter()
            .map(|x| x.1)
            .collect::<Vec<primitives::Transaction>>()
            .into_iter()
            .filter(|x| {
                let asset_ids = x.asset_ids();
                if let Ok(assets) = self.database.get_assets(asset_ids.clone()) {
                    assets.len() == asset_ids.len()
                } else {
                    false
                }
            })
            .collect::<Vec<primitives::Transaction>>();

        let transaction_chunks = primitive_transactions.chunks(300);

        for chunk in transaction_chunks {
            let transactions = chunk
                .to_vec()
                .clone()
                .into_iter()
                .map(storage::models::Transaction::from_primitive)
                .collect::<Vec<storage::models::Transaction>>();

            let transaction_addresses = chunk
                .to_vec()
                .clone()
                .into_iter()
                .flat_map(storage::models::TransactionAddresses::from_primitive)
                .collect::<Vec<storage::models::TransactionAddresses>>();

            if transactions.is_empty() || transaction_addresses.is_empty() {
                return Ok(primitive_transactions.len());
            }

            self.database.add_transactions(transactions.clone(), transaction_addresses.clone())?;
        }

        Ok(primitive_transactions.len())
    }
}
