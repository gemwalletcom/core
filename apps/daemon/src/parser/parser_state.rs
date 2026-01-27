use std::error::Error;

use cacher::{CacheKey, CacherClient};
use primitives::Chain;
use storage::{Database, models::ParserStateRow};

pub struct ParserStateService {
    chain: Chain,
    database: Database,
    cacher: CacherClient,
}

impl ParserStateService {
    pub fn new(chain: Chain, database: Database, cacher: CacherClient) -> Self {
        Self { chain, database, cacher }
    }

    pub async fn init(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let state = self.get_state()?;
        let current_key = CacheKey::ParserCurrentBlock(self.chain.as_ref());
        let latest_key = CacheKey::ParserLatestBlock(self.chain.as_ref());

        if self.cacher.get_i64(&current_key.key()).await?.is_none() {
            self.cacher.set_i64(&current_key.key(), state.current_block, current_key.ttl()).await?;
        }
        if self.cacher.get_i64(&latest_key.key()).await?.is_none() {
            self.cacher.set_i64(&latest_key.key(), state.latest_block, latest_key.ttl()).await?;
        }
        Ok(())
    }

    pub fn get_state(&self) -> Result<ParserStateRow, Box<dyn Error + Send + Sync>> {
        Ok(self.database.parser_state()?.get_parser_state(self.chain.as_ref())?)
    }

    pub async fn get_current_block(&self) -> i64 {
        let key = CacheKey::ParserCurrentBlock(self.chain.as_ref());
        self.cacher.get_i64(&key.key()).await.unwrap_or(None).unwrap_or(0)
    }

    pub async fn get_latest_block(&self) -> i64 {
        let key = CacheKey::ParserLatestBlock(self.chain.as_ref());
        self.cacher.get_i64(&key.key()).await.unwrap_or(None).unwrap_or(0)
    }

    pub async fn set_current_block(&self, block: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let key = CacheKey::ParserCurrentBlock(self.chain.as_ref());
        self.cacher.set_i64(&key.key(), block, key.ttl()).await
    }

    pub async fn set_latest_block(&self, block: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let key = CacheKey::ParserLatestBlock(self.chain.as_ref());
        self.cacher.set_i64(&key.key(), block, key.ttl()).await
    }

    pub async fn persist_state(&self) {
        let current_key = CacheKey::ParserCurrentBlock(self.chain.as_ref());
        let latest_key = CacheKey::ParserLatestBlock(self.chain.as_ref());

        if let Ok(Some(current)) = self.cacher.get_i64(&current_key.key()).await {
            let _ = self.database.parser_state().ok().and_then(|mut c| c.set_parser_state_current_block(self.chain.as_ref(), current).ok());
        }
        if let Ok(Some(latest)) = self.cacher.get_i64(&latest_key.key()).await {
            let _ = self.database.parser_state().ok().and_then(|mut c| c.set_parser_state_latest_block(self.chain.as_ref(), latest).ok());
        }
    }
}
