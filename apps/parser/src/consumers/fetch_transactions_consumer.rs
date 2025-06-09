use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use primitives::{Chain, Transaction};
use settings_chain::ChainProviders;
use storage::DatabaseClient;
use streamer::{consumer::MessageConsumer, ChainAddressPayload, StreamProducer, StreamProducerQueue, TransactionsPayload};
use tokio::sync::Mutex;

pub struct FetchTransactionsConsumer {
    pub database: Arc<Mutex<DatabaseClient>>,
    pub providers: ChainProviders,
    pub producer: StreamProducer,
}

impl FetchTransactionsConsumer {
    pub fn new(database: Arc<Mutex<DatabaseClient>>, providers: ChainProviders, producer: StreamProducer) -> Self {
        Self { database, providers, producer }
    }

    pub async fn process_result(&self, chain: Chain, transactions: Vec<Transaction>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.producer
            .publish_transactions(TransactionsPayload::new(chain, vec![], transactions.clone()))
            .await
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchTransactionsConsumer {
    async fn process(&mut self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        for value in payload.values.clone() {
            match self.providers.get_transactions_by_address(value.chain, value.address.clone()).await {
                Ok(transactions) => {
                    if let Err(e) = self.process_result(value.chain, transactions).await {
                        println!("fetch transactions error, chain: {}, address: {}, error: {:?}", value.chain, value.address, e)
                    }
                }
                Err(e) => {
                    println!("fetch transactions error, chain: {}, address: {}, error: {:?}", value.chain, value.address, e)
                }
            }
        }
        Ok(payload.values.len())
    }
}
