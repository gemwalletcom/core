use std::error::Error;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use cacher::{CacheKey, CacherClient};
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
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

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut updated = 0;
        for chain in Chain::all() {
            if !self.has_pending_transactions(chain).await? {
                continue;
            }
            updated += self.update_chain(chain).await?;
        }

        Ok(updated)
    }

    async fn update_chain(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let pending_key = CacheKey::PendingTransactions(chain.as_ref());
        let identifiers = self.cacher.sorted_set_range_with_scores(&pending_key.key(), 0, -1).await?;

        if identifiers.is_empty() {
            return Ok(0);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64();
        let mut count = 0;

        for (identifier, expires_at) in identifiers {
            if self.process_identifier(chain, &identifier, expires_at, now).await? {
                count += self.remove_pending_transaction(chain, &identifier).await?;
            }
        }

        Ok(count)
    }

    async fn process_identifier(&self, chain: Chain, identifier: &str, expires_at: f64, now: f64) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if expires_at <= now {
            info_with_fields!("pending transaction expired", chain = chain.as_ref(), identifier = identifier);
            return Ok(true);
        }

        let start = Instant::now();
        match self.providers.get_transaction_by_hash(chain, identifier.to_string()).await {
            Ok(Some(transaction)) => {
                info_with_fields!(
                    "pending transaction fetch success",
                    chain = chain.as_ref(),
                    identifier = identifier,
                    elapsed = DurationMs(start.elapsed())
                );
                self.stream_producer
                    .publish_transactions(TransactionsPayload::new(chain, vec![], vec![transaction]))
                    .await?;
                Ok(true)
            }
            Ok(None) => {
                info_with_fields!(
                    "pending transaction not fetched",
                    chain = chain.as_ref(),
                    identifier = identifier,
                    elapsed = DurationMs(start.elapsed())
                );
                Ok(false)
            }
            Err(err) => {
                error_with_fields!(
                    "pending transaction fetch failed",
                    &*err,
                    chain = chain.as_ref(),
                    identifier = identifier,
                    elapsed = DurationMs(start.elapsed())
                );
                Ok(false)
            }
        }
    }

    async fn remove_pending_transaction(&self, chain: Chain, identifier: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.cacher
            .remove_from_sorted_set_cached(CacheKey::PendingTransactions(chain.as_ref()), &[identifier.to_string()])
            .await
    }

    async fn has_pending_transactions(&self, chain: Chain) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let pending_key = CacheKey::PendingTransactions(chain.as_ref());
        let pending_count = self.cacher.sorted_set_card(&pending_key.key()).await?;
        Ok(pending_count > 0)
    }
}
