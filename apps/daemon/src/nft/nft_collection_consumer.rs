use std::error::Error;

use async_trait::async_trait;
use streamer::{consumer::MessageConsumer, FetchNFTCollectionPayload};

pub struct UpdateNftCollectionConsumer {}

impl UpdateNftCollectionConsumer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MessageConsumer<FetchNFTCollectionPayload, usize> for UpdateNftCollectionConsumer {
    async fn should_process(&mut self, _payload: FetchNFTCollectionPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }
    async fn process(&mut self, _payload: FetchNFTCollectionPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(0)
    }
}
