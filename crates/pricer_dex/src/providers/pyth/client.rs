use super::model::{HermesResponse, Price};
use reqwest::Client;

pub struct PythClient {
    hermes_url: String,
    client: Client,
}

impl PythClient {
    pub fn new(hermes_url: &str) -> Self {
        Self {
            hermes_url: hermes_url.to_string(),
            client: Client::new(),
        }
    }

    pub async fn get_asset_prices(&self, price_ids: Vec<String>) -> Result<Vec<Price>, Box<dyn std::error::Error + Send + Sync>> {
        const CHUNK_SIZE: usize = 5;
        let mut all_prices = Vec::new();

        for chunk in price_ids.chunks(CHUNK_SIZE) {
            let url = format!("{}/v2/updates/price/latest", self.hermes_url);

            let mut request = self.client.get(&url);
            for id in chunk {
                request = request.query(&[("ids[]", id)]);
            }

            let response = request.send().await?;
            let hermes_response: HermesResponse = response.json().await?;

            let prices: Vec<Price> = hermes_response
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

    pub async fn get_price(&self, price_id: &str) -> Result<Price, Box<dyn std::error::Error + Send + Sync>> {
        let prices = self.get_asset_prices(vec![price_id.to_string()]).await?;
        prices.into_iter().next().ok_or_else(|| "No price found".into())
    }
}
