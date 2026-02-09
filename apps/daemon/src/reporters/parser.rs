use cacher::{CacheKey, CacherClient};
use primitives::{Chain, ParserStatus};

pub struct ParserReporter {
    chain: Chain,
    cacher: CacherClient,
}

impl ParserReporter {
    pub fn new(chain: Chain, cacher: CacherClient) -> Self {
        Self { chain, cacher }
    }

    pub async fn error(&self, error: &str) {
        let cache_key = CacheKey::ParserStatus(self.chain.as_ref());
        let key = cache_key.key();
        let mut status = self.cacher.get_value::<ParserStatus>(&key).await.unwrap_or_default();

        super::record_error(&mut status.errors, error);

        let _ = self.cacher.set_cached(cache_key, &status).await;
    }
}
