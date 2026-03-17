use std::error::Error;
use std::sync::Arc;

use cacher::CacheKey;
use cacher::CacherClient;
use chain_traits::TransactionsRequest;
use gem_tracing::error_with_fields;
use primitives::Chain;
use settings_chain::ChainProviders;
use streamer::steam_producer_queue::StreamProducerQueue;
use streamer::{StreamProducer, TransactionsPayload};

pub struct PerpetualPositionObserver {
    chain: Chain,
    providers: Arc<ChainProviders>,
    cacher: CacherClient,
    stream_producer: StreamProducer,
}

impl PerpetualPositionObserver {
    pub fn new(chain: Chain, providers: Arc<ChainProviders>, cacher: CacherClient, stream_producer: StreamProducer) -> Self {
        Self {
            chain,
            providers,
            cacher,
            stream_producer,
        }
    }

    pub async fn observe(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let key = CacheKey::PerpetualActiveAddresses(self.chain.as_ref());
        let addresses = self.cacher.get_set_members_cached(vec![key.key()]).await?;

        for address in &addresses {
            if let Err(error) = self.observe_address(address).await {
                error_with_fields!("perpetual_observer", &*error, chain = self.chain.as_ref(), address = address);
            }
        }

        Ok(addresses.len())
    }

    async fn observe_address(&self, address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let checkpoint = CacheKey::PerpetualObserverCheckpoint(self.chain.as_ref(), address);
        let checkpoint_key = checkpoint.key();
        let now = chrono::Utc::now().timestamp() as u64;
        let from_timestamp: u64 = self.cacher.get_value_optional(&checkpoint_key).await?.unwrap_or(now);

        let request = TransactionsRequest::new(address.to_string()).with_from_timestamp(Some(from_timestamp));
        let transactions = self.providers.get_transactions_by_address(self.chain, request).await?;

        if !transactions.is_empty() {
            let payload = TransactionsPayload::new(self.chain, vec![0], transactions);
            self.stream_producer.publish_transactions(payload).await?;
        }

        self.cacher.set_value_with_ttl(&checkpoint_key, now.to_string(), checkpoint.ttl()).await?;

        Ok(())
    }
}
