use async_trait::async_trait;
use std::error::Error;

use streamer::FetchPricesPayload;
use streamer::consumer::MessageConsumer;

use crate::worker::pricer::price_updater::PriceUpdater;

pub struct FetchPricesConsumer {
    price_updater: PriceUpdater,
}

impl FetchPricesConsumer {
    pub fn new(price_updater: PriceUpdater) -> Self {
        Self { price_updater }
    }
}

#[async_trait]
impl MessageConsumer<FetchPricesPayload, usize> for FetchPricesConsumer {
    async fn process(&self, payload: FetchPricesPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.price_updater.update_prices(payload.price_ids).await
    }

    async fn should_process(&self, payload: FetchPricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(!payload.price_ids.is_empty())
    }
}
