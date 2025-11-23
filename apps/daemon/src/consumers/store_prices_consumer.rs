use async_trait::async_trait;
use pricer::PriceClient;
use primitives::AssetId;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use storage::Database;
use storage::models::Price;
use streamer::{PricesPayload, consumer::MessageConsumer};

pub struct StorePricesConsumer {
    pub database: Database,
    pub price_client: PriceClient,
    pub ttl_seconds: i64,
}

impl StorePricesConsumer {
    pub fn new(database: Database, price_client: PriceClient, ttl_seconds: i64) -> Self {
        Self {
            database,
            price_client,
            ttl_seconds,
        }
    }

    async fn update_prices_cache(&self, updated_prices: Vec<Price>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let price_ids: HashSet<_> = updated_prices.iter().map(|p| &p.id).collect();

        let prices_assets_map: HashMap<_, Vec<_>> = self
            .price_client
            .get_prices_assets()?
            .into_iter()
            .filter(|price_asset| price_ids.contains(&price_asset.price_id))
            .fold(HashMap::new(), |mut map, price_asset| {
                map.entry(price_asset.price_id).or_default().push(price_asset.asset_id);
                map
            });

        let prices: Vec<_> = updated_prices
            .into_iter()
            .flat_map(|price| {
                prices_assets_map
                    .get(&price.id)
                    .into_iter()
                    .flatten()
                    .filter_map(move |asset_id| AssetId::new(asset_id).map(|id| price.as_price_asset_info(id)))
            })
            .collect();

        self.price_client.set_cache_prices(prices, self.ttl_seconds).await
    }
}

#[async_trait]
impl MessageConsumer<PricesPayload, usize> for StorePricesConsumer {
    async fn should_process(&self, _payload: PricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: PricesPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let prices: Vec<Price> = payload.prices.iter().map(|p| Price::from_price_data(p.clone())).collect();
        self.database.client()?.prices().set_prices(prices.clone())?;

        self.update_prices_cache(prices).await?;

        Ok(payload.prices.len())
    }
}
