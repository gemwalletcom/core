use std::error::Error;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use cacher::{CacheKey, CacherClient};
use gem_tracing::error_with_fields;
use primitives::Chain;
use settings_chain::ChainProviders;
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};

pub struct PendingTransactionsUpdater {
    providers: Arc<ChainProviders>,
    cacher: CacherClient,
    stream_producer: StreamProducer,
}

impl PendingTransactionsUpdater {
    pub fn new(providers: Arc<ChainProviders>, cacher: CacherClient, stream_producer: StreamProducer) -> Self {
        Self {
            providers,
            cacher,
            stream_producer,
        }
    }

    pub async fn update(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let pending_key = CacheKey::PendingTransactions(chain.as_ref());
        let identifiers = self.cacher.sorted_set_range_with_scores(&pending_key.key(), 0, -1).await?;

        if identifiers.is_empty() {
            return Ok(0);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64();
        let mut count = 0;

        for (identifier, expires_at) in identifiers {
            if expires_at <= now {
                count += self
                    .cacher
                    .remove_from_sorted_set_cached(CacheKey::PendingTransactions(chain.as_ref()), std::slice::from_ref(&identifier))
                    .await?;
                continue;
            }

            let transaction = match self.providers.get_transaction_by_hash(chain, identifier.clone()).await {
                Ok(transaction) => transaction,
                Err(err) => {
                    error_with_fields!("pending transaction fetch failed", &*err, chain = chain.as_ref(), identifier = identifier.as_str());
                    None
                }
            };

            if let Some(transaction) = transaction {
                self.stream_producer
                    .publish_transactions(TransactionsPayload::new(chain, vec![], vec![transaction]))
                    .await?;
                count += self
                    .cacher
                    .remove_from_sorted_set_cached(CacheKey::PendingTransactions(chain.as_ref()), std::slice::from_ref(&identifier))
                    .await?;
            }
        }

        Ok(count)
    }
}
