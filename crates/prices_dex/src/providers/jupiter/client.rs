use std::error::Error;

use super::model::{JupiterPriceResponse, Price, TopTokensResponse, VerifiedTokensResponse};
use gem_client::{ClientExt, ReqwestClient};

pub struct JupiterClient {
    client: ReqwestClient,
}

impl JupiterClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_top_tokens(&self, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let query = vec![("limit".to_string(), limit.to_string())];
        let response = self.client.get_with_query::<TopTokensResponse>("/tokens/v1/top-tokens", &query).await?;
        Ok(response.tokens)
    }

    pub async fn get_verified_tokens(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let query = vec![("query".to_string(), "verified".to_string())];
        let response = self.client.get_with_query::<VerifiedTokensResponse>("/tokens/v2/tag", &query).await?;
        Ok(response.into_iter().map(|token| token.id).collect())
    }

    pub async fn get_asset_prices(&self, token_addresses: Vec<String>) -> Result<Vec<Price>, Box<dyn Error + Send + Sync>> {
        const CHUNK_SIZE: usize = 50;
        let mut all_prices = Vec::new();

        for chunk in token_addresses.chunks(CHUNK_SIZE) {
            let ids = chunk.join(",");
            let query = vec![("ids".to_string(), ids)];

            let response = self.client.get_with_query::<JupiterPriceResponse>("/price/v3", &query).await?;

            let prices: Vec<Price> = chunk
                .iter()
                .filter_map(|address| {
                    response.get(address).map(|token_data| Price {
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
