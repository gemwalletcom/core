use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use primitives::{TransactionId, chain_transaction_timeout_seconds};
use streamer::consumer::MessageConsumer;

pub struct StorePendingTransactionsConsumer {
    cacher: CacherClient,
}

impl StorePendingTransactionsConsumer {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }
}

#[async_trait]
impl MessageConsumer<TransactionId, usize> for StorePendingTransactionsConsumer {
    async fn should_process(&self, _payload: TransactionId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: TransactionId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .saturating_add(u64::from(chain_transaction_timeout_seconds(payload.chain))) as f64;
        let key = CacheKey::PendingTransactions(payload.chain.as_ref());
        self.cacher.add_to_sorted_set_cached(key, &[(payload.hash, expires_at)]).await?;
        Ok(1)
    }
}
