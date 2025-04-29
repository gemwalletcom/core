use chrono::{Duration, Utc};
use coingecko::{CoinGeckoClient, CoinMarket};
use pricer::PriceClient;
use primitives::DEFAULT_FIAT_CURRENCY;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use storage::models::price::PriceCache;
use storage::models::Price;

pub struct PriceUpdater {
    coin_gecko_client: CoinGeckoClient,
    price_client: PriceClient,
}

pub enum UpdatePrices {
    Top,
    High,
    Low,
}

const MAX_MARKETS_PER_PAGE: usize = 250;

impl PriceUpdater {
    pub fn new(price_client: PriceClient, coin_gecko_client: CoinGeckoClient) -> Self {
        Self {
            coin_gecko_client,
            price_client,
        }
    }

    pub async fn update_prices_type(&mut self, update_type: UpdatePrices) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
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
            .map(|x| Self::map_price_for_market(x.clone()))
            .filter(|x| x.last_updated_at.is_some())
            .collect::<HashSet<Price>>()
            .into_iter()
            .collect::<Vec<Price>>()
    }

    pub async fn update_prices(&mut self, ids: Vec<String>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let ids_chunks = ids.chunks(250);
        for ids in ids_chunks {
            let coin_markets = self.coin_gecko_client.get_coin_markets_ids(ids.to_vec(), MAX_MARKETS_PER_PAGE).await?;
            let prices = Self::map_coin_markets(coin_markets);
            self.price_client.set_prices(prices)?;
        }
        Ok(ids.len())
    }

    pub async fn update_prices_pages(&mut self, pages: usize) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let coin_markets = self.coin_gecko_client.get_all_coin_markets(MAX_MARKETS_PER_PAGE, pages).await?;
        let prices = Self::map_coin_markets(coin_markets);
        self.price_client.set_prices(prices)
    }

    pub async fn update_fiat_rates_cache(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let rates = self.price_client.get_fiat_rates()?;
        self.price_client.set_cache_fiat_rates(rates.clone()).await?;
        Ok(rates.len())
    }

    pub async fn update_prices_cache(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let (prices_assets, prices, rates) = (
            self.price_client.get_prices_assets()?,
            self.price_client.get_prices()?,
            self.price_client.get_fiat_rates()?,
        );

        let prices_assets_map: HashMap<String, HashSet<String>> = prices_assets.into_iter().fold(HashMap::new(), |mut map, price_asset| {
            map.entry(price_asset.price_id.clone()).or_default().insert(price_asset.asset_id);
            map
        });

        let base_rate = rates
            .iter()
            .find(|x| x.symbol == DEFAULT_FIAT_CURRENCY)
            .map(|x| x.rate)
            .ok_or("base rate not found")?;

        for rate in rates.iter() {
            let prices: Vec<PriceCache> = prices
                .clone()
                .into_iter()
                .flat_map(|price| {
                    let new_price = Price::for_rate(price, base_rate, rate.clone());

                    if let Some(asset_ids) = prices_assets_map.get(new_price.id.as_str()) {
                        return asset_ids
                            .iter()
                            .map(|asset_id| PriceCache {
                                price: new_price.clone(),
                                asset_id: asset_id.clone(),
                            })
                            .collect::<Vec<PriceCache>>();
                    }
                    vec![]
                })
                .collect();

            if prices.is_empty() {
                continue;
            }
            match self.price_client.set_cache_prices(rate.symbol.as_str(), prices).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error setting cache prices for {}: {}", rate.symbol, e);
                }
            }
        }

        Ok(prices.len())
    }

    pub async fn update_fiat_rates(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let rates = self.coin_gecko_client.get_fiat_rates().await?;
        self.price_client.set_fiat_rates(rates).await
    }

    pub async fn clean_outdated_assets(&mut self, seconds: u64) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let time = Utc::now() - Duration::seconds(seconds as i64);
        self.price_client.delete_prices_updated_at_before(time.naive_utc())
    }

    fn map_price_for_market(market: CoinMarket) -> Price {
        Price::new(
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
            market.last_updated.map(|x| x.naive_local()),
        )
    }
}
