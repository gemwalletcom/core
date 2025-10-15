use super::model::{JupiterPriceResponse, Price, TopTokensResponse, VerifiedTokensResponse};
use reqwest::Client;
use std::error::Error;

pub struct JupiterClient {
    base_url: String,
    client: Client,
}

impl JupiterClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }

    pub async fn get_top_tokens(&self, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/tokens/v1/top-tokens", self.base_url);
        let response = self.client.get(&url).query(&[("limit", limit.to_string())]).send().await?;

        let top_tokens: TopTokensResponse = response.json().await?;
        Ok(top_tokens.tokens)
    }

    pub async fn get_verified_tokens(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/tokens/v2/tag", self.base_url);
        let response = self.client.get(&url).query(&[("query", "verified")]).send().await?;

        let verified_tokens: VerifiedTokensResponse = response.json().await?;
        Ok(verified_tokens.into_iter().map(|token| token.id).collect())
    }

    pub async fn get_asset_prices(&self, token_addresses: Vec<String>) -> Result<Vec<Price>, Box<dyn Error + Send + Sync>> {
        const CHUNK_SIZE: usize = 50;
        let mut all_prices = Vec::new();

        for chunk in token_addresses.chunks(CHUNK_SIZE) {
            let url = format!("{}/price/v3", self.base_url);
            let ids = chunk.join(",");

            let response = self.client.get(&url).query(&[("ids", ids)]).send().await?;

            let response_text = response.text().await?;
            let jupiter_response: JupiterPriceResponse = serde_json::from_str(&response_text).map_err(|e| {
                eprintln!("Failed to parse Jupiter response: {}", e);
                eprintln!("Response text: {}", response_text);
                e
            })?;

            let prices: Vec<Price> = chunk
                .iter()
                .filter_map(|address| {
                    jupiter_response.get(address).map(|token_data| Price {
                        price: token_data.usd_price,
                        price_change_24h: token_data.price_change_24h,
                    })
                })
                .collect();

            all_prices.extend(prices);
        }

        Ok(all_prices)
    }

    pub async fn get_price(&self, token_address: &str) -> Result<Price, Box<dyn Error + Send + Sync>> {
        let prices = self.get_asset_prices(vec![token_address.to_string()]).await?;
        prices.into_iter().next().ok_or_else(|| "No price found".into())
    }
}
