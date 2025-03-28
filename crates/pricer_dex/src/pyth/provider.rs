use primitives::AssetPrice;

use crate::PriceChainProvider;
use async_trait::async_trait;

use super::{client::PythClient, mapper::price_account_for_chain};

pub struct PythProvider {
    pub pyth_client: PythClient,
}

#[async_trait]
impl PriceChainProvider for PythProvider {
    async fn get_chain_prices(&self, chains: Vec<primitives::Chain>) -> Result<Vec<AssetPrice>, Box<dyn std::error::Error + Send + Sync>> {
        let price_ids = chains.iter().map(|x| price_account_for_chain(*x).to_string()).collect();
        let prices = self.pyth_client.get_asset_prices(price_ids).await?;

        Ok(chains
            .into_iter()
            .zip(prices.into_iter())
            .map(|(chain, price)| AssetPrice {
                asset_id: chain.as_ref().to_string(),
                price: price.price,
                price_change_percentage_24h: 0.0,
            })
            .collect())
    }
}
