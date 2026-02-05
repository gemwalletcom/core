use std::error::Error;

use async_trait::async_trait;
use settings_chain::ChainProviders;
use streamer::{FetchBlocksPayload, StreamProducer, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};

pub struct FetchBlocksConsumer {
    pub providers: ChainProviders,
    pub stream_producer: StreamProducer,
}

impl FetchBlocksConsumer {
    pub fn new(providers: ChainProviders, stream_producer: StreamProducer) -> Self {
        Self { providers, stream_producer }
    }
}

#[async_trait]
impl MessageConsumer<FetchBlocksPayload, usize> for FetchBlocksConsumer {
    async fn should_process(&self, _payload: FetchBlocksPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }
    async fn process(&self, payload: FetchBlocksPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let blocks = vec![payload.block];
        let transactions = self.providers.get_transactions_in_blocks(payload.chain, blocks.clone()).await?;
        let payload = TransactionsPayload::new(payload.chain, blocks, transactions.clone());
        self.stream_producer.publish_transactions(payload).await?;
        Ok(transactions.len())
    }
}
