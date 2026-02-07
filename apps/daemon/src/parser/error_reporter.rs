use cacher::{CacheKey, CacherClient};
use primitives::{Chain, ParserError, ParserStatus};

pub struct ErrorReporter {
    chain: Chain,
    cacher: CacherClient,
}

impl ErrorReporter {
    pub fn new(chain: Chain, cacher: CacherClient) -> Self {
        Self { chain, cacher }
    }

    pub async fn error(&self, error: &str) {
        let cache_key = CacheKey::ParserStatus(self.chain.as_ref());
        let key = cache_key.key();
        let mut status = match self.cacher.get_value::<ParserStatus>(&key).await {
            Ok(status) => status,
            Err(_) => ParserStatus::default(),
        };
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

        let message = if error.len() > 200 { &error[..200] } else { error };

        if let Some(entry) = status.errors.iter_mut().find(|e| e.message == message) {
            entry.count += 1;
            entry.timestamp = timestamp;
        } else {
            status.errors.push(ParserError {
                message: message.to_string(),
                count: 1,
                timestamp,
            });
        }

        let _ = self.cacher.set_cached(cache_key, &status).await;
    }
}
