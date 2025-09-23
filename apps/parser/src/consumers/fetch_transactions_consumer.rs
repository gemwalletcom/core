use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use cacher::CacherClient;
use primitives::{Chain, Transaction};
use settings_chain::ChainProviders;
use storage::DatabaseClient;
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};
use tokio::sync::Mutex;

pub struct FetchTransactionsConsumer {
    pub database: Arc<Mutex<DatabaseClient>>,
    pub providers: ChainProviders,
    pub producer: StreamProducer,
    pub cacher: CacherClient,
}

impl FetchTransactionsConsumer {
    pub fn new(database: Arc<Mutex<DatabaseClient>>, providers: ChainProviders, producer: StreamProducer, cacher: CacherClient) -> Self {
        Self {
            database,
            providers,
            producer,
            cacher,
        }
    }

    pub async fn process_result(&self, chain: Chain, transactions: Vec<Transaction>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.producer
            .publish_transactions(TransactionsPayload::new(chain, vec![], transactions.clone()))
            .await
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchTransactionsConsumer {
    async fn should_process(&mut self, payload: ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher
            .can_process_now(&format!("fetch_transactions:{}:{}", payload.value.chain, payload.value.address), 30 * 86400)
            .await
    }
    async fn process(&mut self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions = self
            .providers
            .get_transactions_by_address(payload.value.chain, payload.value.address.clone())
            .await?;
        let _ = self.process_result(payload.value.chain, transactions.clone()).await;
        Ok(transactions.len())
    }
}
