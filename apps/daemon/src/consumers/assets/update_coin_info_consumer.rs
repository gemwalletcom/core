use std::error::Error;

use async_trait::async_trait;
use streamer::UpdateCoinInfoPayload;

use crate::worker::assets::asset_updater::AssetProcessor;

pub struct UpdateCoinInfoConsumer {
    processor: AssetProcessor,
}

impl UpdateCoinInfoConsumer {
    pub fn new(processor: AssetProcessor) -> Self {
        Self { processor }
    }
}

#[async_trait]
impl streamer::consumer::MessageConsumer<UpdateCoinInfoPayload, usize> for UpdateCoinInfoConsumer {
    async fn should_process(&self, _payload: UpdateCoinInfoPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: UpdateCoinInfoPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.processor.process_coin_update(&payload.coin_id).await
    }
}
