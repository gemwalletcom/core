use std::error::Error;

use super::model::{HermesResponse, Price, PriceFeed};
use gem_client::{ClientExt, ReqwestClient};

pub struct PythClient {
    client: ReqwestClient,
}

impl PythClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_price_feeds(&self) -> Result<Vec<PriceFeed>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/v2/price_feeds").await?)
    }

    pub async fn get_asset_prices(&self, price_ids: Vec<String>) -> Result<Vec<Price>, Box<dyn Error + Send + Sync>> {
        const CHUNK_SIZE: usize = 5;
        let mut all_prices = Vec::new();

        for chunk in price_ids.chunks(CHUNK_SIZE) {
            let query: Vec<(String, String)> = chunk.iter().map(|id| ("ids[]".to_string(), id.clone())).collect();

            let response = self.client.get_with_query::<HermesResponse>("/v2/updates/price/latest", &query).await?;

            let prices: Vec<Price> = response
                .parsed
                .into_iter()
                .map(|feed| {
                    let scaled_price = feed.price.price as f64 * 10f64.powi(feed.price.expo);
                    Price { price: scaled_price }
                })
                .collect();

            all_prices.extend(prices);
        }

        Ok(all_prices)
    }

    pub async fn get_price(&self, price_id: &str) -> Result<Price, Box<dyn Error + Send + Sync>> {
        let prices = self.get_asset_prices(vec![price_id.to_string()]).await?;
        prices.into_iter().next().ok_or_else(|| "No price found".into())
    }
}
