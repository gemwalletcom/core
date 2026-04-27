use std::error::Error;

use async_trait::async_trait;
use gem_tracing::info_with_fields;
use pricer::{PriceClient, PriceProviders};
use streamer::{FetchPricesPayload, consumer::MessageConsumer};

pub struct FetchPricesConsumer {
    pub price_client: PriceClient,
    pub providers: PriceProviders,
}

#[async_trait]
impl MessageConsumer<FetchPricesPayload, usize> for FetchPricesConsumer {
    async fn should_process(&self, _payload: FetchPricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: FetchPricesPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let count = match &payload {
            FetchPricesPayload::AssetId(asset_id) => self.price_client.add_prices_for_asset_id(&self.providers, asset_id).await?,
            FetchPricesPayload::PriceId(price_id) => self.price_client.add_prices_for_price_id(&self.providers, price_id).await?,
        };
        let payload_str = payload.to_string();
        info_with_fields!("fetch prices", payload = payload_str.as_str(), count = count);
        Ok(count)
    }
}
