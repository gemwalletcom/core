use std::error::Error;

use async_trait::async_trait;
use storage::DatabaseClient;
use streamer::{consumer::MessageConsumer, AssetsPayload};

pub struct AssetsConsumer {
    pub database: DatabaseClient,
}

#[async_trait]
impl MessageConsumer<AssetsPayload, usize> for AssetsConsumer {
    async fn process(&mut self, _payload: AssetsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(0)
    }
}
