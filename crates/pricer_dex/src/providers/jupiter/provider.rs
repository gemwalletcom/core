use async_trait::async_trait;
use chrono::Utc;
use primitives::{AssetId, AssetPrice, Chain};

use crate::PriceChainAssetsProvider;

use super::{client::JupiterClient, mapper::map_asset_id_to_token};

pub struct JupiterProvider {
    pub jupiter_client: JupiterClient,
}

impl JupiterProvider {
    pub fn new(base_url: &str) -> Self {
        Self {
            jupiter_client: JupiterClient::new(base_url),
        }
    }
}

#[async_trait]
impl PriceChainAssetsProvider for JupiterProvider {
    async fn get_asset_prices(&self, chain: Chain, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, Box<dyn std::error::Error + Send + Sync>> {
        if chain != Chain::Solana {
            return Ok(vec![]);
        }

        let token_addresses: Vec<String> = asset_ids.iter().filter_map(|id| map_asset_id_to_token(id)).collect();

        if token_addresses.is_empty() {
            return Ok(vec![]);
        }

        let prices = self.jupiter_client.get_asset_prices(token_addresses.clone()).await?;

        Ok(asset_ids
            .into_iter()
            .zip(prices)
            .map(|(asset_id, price)| AssetPrice {
                asset_id,
                price: price.price,
                price_change_percentage_24h: price.price_change_24h,
                updated_at: Utc::now(),
            })
            .collect())
    }
}
