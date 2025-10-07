use std::{error::Error, vec};

use coingecko::{CoinGeckoClient, model::Global};
use pricer::MarketsClient;
use primitives::{AssetTag, Chain, MarketDominance, Markets};

pub struct MarketsUpdater {
    markets_client: MarketsClient,
    coin_gecko_client: CoinGeckoClient,
}

impl MarketsUpdater {
    pub fn new(markets_client: MarketsClient, coin_gecko_client: CoinGeckoClient) -> Self {
        Self {
            markets_client,
            coin_gecko_client,
        }
    }

    pub async fn update_markets(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let global = self.coin_gecko_client.get_global().await?;
        let trending = self.coin_gecko_client.get_search_trending().await?;
        let top_gainers_losers = self.coin_gecko_client.get_top_gainers_losers().await?;

        let trending = self.markets_client.get_asset_ids_for_prices_ids(trending.get_coins_ids()).await?;
        let gainers = self.markets_client.get_asset_ids_for_prices_ids(top_gainers_losers.get_gainers_ids()).await?;
        let losers = self.markets_client.get_asset_ids_for_prices_ids(top_gainers_losers.get_losers_ids()).await?;
        let dominance = self.dominance(global.clone());

        let _ = self.markets_client.set_asset_ids_for_tag(AssetTag::Trending, trending);
        let _ = self.markets_client.set_asset_ids_for_tag(AssetTag::Gainers, gainers);
        let _ = self.markets_client.set_asset_ids_for_tag(AssetTag::Losers, losers);

        let assets = self.markets_client.get_market_assets()?;

        let markets = Markets {
            market_cap: global.total_market_cap.usd as f32,
            market_cap_change_percentage_24h: global.market_cap_change_percentage_24h_usd as f32,
            assets,
            dominance,
            total_volume_24h: global.total_volume.usd as f32,
        };

        self.markets_client.set_markets(markets).await?;

        Ok(1)
    }

    fn dominance(&self, global: Global) -> Vec<MarketDominance> {
        vec![
            MarketDominance {
                asset_id: Chain::Bitcoin.to_string(),
                dominance: *global.market_cap_percentage.get("btc").unwrap_or(&0.0) as f32,
            },
            MarketDominance {
                asset_id: Chain::Ethereum.to_string(),
                dominance: *global.market_cap_percentage.get("eth").unwrap_or(&0.0) as f32,
            },
            MarketDominance {
                asset_id: Chain::Solana.to_string(),
                dominance: *global.market_cap_percentage.get("sol").unwrap_or(&0.0) as f32,
            },
            MarketDominance {
                asset_id: Chain::SmartChain.to_string(),
                dominance: *global.market_cap_percentage.get("bnb").unwrap_or(&0.0) as f32,
            },
        ]
    }
}
