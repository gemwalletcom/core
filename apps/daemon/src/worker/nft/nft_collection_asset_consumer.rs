use std::error::Error;

use async_trait::async_trait;
use streamer::{FetchNFTCollectionAssetPayload, consumer::MessageConsumer};

pub struct UpdateNftCollectionAssetsConsumer {}

impl UpdateNftCollectionAssetsConsumer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MessageConsumer<FetchNFTCollectionAssetPayload, usize> for UpdateNftCollectionAssetsConsumer {
    async fn should_process(&mut self, _payload: FetchNFTCollectionAssetPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }
    async fn process(&mut self, _payload: FetchNFTCollectionAssetPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(0)
    }
}
