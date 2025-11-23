use chrono::{DateTime, Duration, Utc};
use coingecko::{CoinGeckoClient, CoinMarket};
use pricer::PriceClient;
use std::collections::HashSet;
use std::error::Error;
use storage::models::{Chart, Price};
use streamer::{ChartsPayload, PricesPayload, StreamProducer, StreamProducerQueue};

pub struct PriceUpdater {
    coin_gecko_client: CoinGeckoClient,
    price_client: PriceClient,
    stream_producer: StreamProducer,
}

pub enum UpdatePrices {
    Top,
    High,
    Low,
}

const MAX_MARKETS_PER_PAGE: usize = 250;

impl PriceUpdater {
    pub fn new(price_client: PriceClient, coin_gecko_client: CoinGeckoClient, stream_producer: StreamProducer) -> Self {
        Self {
            coin_gecko_client,
            price_client,
            stream_producer,
        }
    }

    pub async fn update_prices_type(&self, update_type: UpdatePrices) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let ids = self.price_client.get_prices_ids()?;
        let asset_ids = match update_type {
            UpdatePrices::Top => ids.into_iter().take(500).collect::<Vec<_>>(),
            UpdatePrices::High => ids.into_iter().take(2500).skip(500).collect::<Vec<_>>(),
            UpdatePrices::Low => ids.into_iter().skip(2500).collect::<Vec<_>>(),
        };
        self.update_prices(asset_ids).await
    }

    fn map_coin_markets(coin_markets: Vec<CoinMarket>) -> Vec<Price> {
        coin_markets
            .into_iter()
            .flat_map(|x| Self::map_price_for_market(x.clone()))
            .collect::<HashSet<Price>>()
            .into_iter()
            .collect::<Vec<Price>>()
    }

    pub async fn update_prices(&self, ids: Vec<String>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let ids_chunks = ids.chunks(MAX_MARKETS_PER_PAGE);
        for ids in ids_chunks {
            let coin_markets = self.coin_gecko_client.get_coin_markets_ids(ids.to_vec(), MAX_MARKETS_PER_PAGE).await?;
            let prices = Self::map_coin_markets(coin_markets);

            let prices_data = prices.iter().map(|p| p.as_price_data()).collect();
            let charts_data = prices.iter().map(|p| Chart::from_price(p.clone()).as_chart_data()).collect();

            self.stream_producer.publish_prices(PricesPayload::new(prices_data)).await?;
            self.stream_producer.publish_charts(ChartsPayload::new(charts_data)).await?;
        }
        Ok(ids.len())
    }

    pub async fn update_fiat_rates(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let rates = self.coin_gecko_client.get_fiat_rates().await?;
        self.price_client.set_fiat_rates(rates).await
    }

    pub async fn clean_outdated_assets(&self, seconds: u64) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let time = Utc::now() - Duration::seconds(seconds as i64);
        self.price_client.delete_prices_updated_at_before(time.naive_utc())
    }

    fn map_price_for_market(market: CoinMarket) -> Option<Price> {
        let last_updated_at = market.last_updated.map(|x: DateTime<Utc>| x.naive_utc())?;
        Some(Price::new(
            market.id,
            market.current_price.unwrap_or_default(),
            market.price_change_percentage_24h.unwrap_or_default(),
            market.ath.unwrap_or_default(),
            market.ath_date.map(|x| x.naive_local()),
            market.atl.unwrap_or_default(),
            market.atl_date.map(|x| x.naive_local()),
            market.market_cap.unwrap_or_default(),
            market.fully_diluted_valuation.unwrap_or_default(),
            market.market_cap_rank.unwrap_or_default(),
            market.total_volume.unwrap_or_default(),
            market.circulating_supply.unwrap_or_default(),
            market.total_supply.unwrap_or_default(),
            market.max_supply.unwrap_or_default(),
            last_updated_at,
        ))
    }
}
