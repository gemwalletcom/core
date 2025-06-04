use std::error::Error;

use async_trait::async_trait;
use gem_chain_rpc::ChainBlockProvider;
use streamer::{consumer::MessageConsumer, FetchBlocksPayload, QueueName, StreamProducer, TransactionsPayload};

pub struct FetchBlocksConsumer {
    pub providers: Vec<Box<dyn ChainBlockProvider>>,
    pub stream_producer: StreamProducer,
}

impl FetchBlocksConsumer {
    pub fn new(providers: Vec<Box<dyn ChainBlockProvider>>, stream_producer: StreamProducer) -> Self {
        Self { providers, stream_producer }
    }
}

#[async_trait]
impl MessageConsumer<FetchBlocksPayload, usize> for FetchBlocksConsumer {
    async fn process(&mut self, payload: FetchBlocksPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.providers.iter().find(|x| x.get_chain() == payload.chain).ok_or("provider not found")?;
        let transactions = provider.get_transactions_in_blocks(payload.blocks.clone()).await?;
        let payload = TransactionsPayload::new(payload.chain, payload.blocks.clone(), transactions.clone());
        self.stream_producer.publish(QueueName::Transactions, &payload).await?;
        Ok(transactions.len())
    }
}
