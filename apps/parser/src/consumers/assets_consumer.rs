use std::error::Error;

use async_trait::async_trait;
use storage::DatabaseClient;
use streamer::{consumer::MessageConsumer, FetchAssetsPayload};

pub struct FetchAssetsConsumer {
    pub database: DatabaseClient,
}

#[async_trait]
impl MessageConsumer<FetchAssetsPayload, usize> for FetchAssetsConsumer {
    async fn process(&mut self, _payload: FetchAssetsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(0)
    }
}
