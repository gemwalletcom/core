use crate::client::PriceClient;
use crate::DEFAULT_FIAT_CURRENCY;
use chrono::{Duration, Utc};
use coingecko::mapper::{get_chain_for_coingecko_platform_id, get_coingecko_market_id_for_chain};
use coingecko::{Coin, CoinGeckoClient, CoinMarket};
use primitives::chain::Chain;
use primitives::AssetId;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::thread;
use storage::models::price::{PriceAsset, PriceCache};
use storage::models::{ChartCoinPrice, FiatRate, Price};

pub struct PriceUpdater {
    coin_gecko_client: CoinGeckoClient,
    price_client: PriceClient,
}

impl PriceUpdater {
    pub fn new(price_client: PriceClient, coin_gecko_client: CoinGeckoClient) -> Self {
        PriceUpdater {
            coin_gecko_client,
            price_client,
        }
    }

    pub async fn get_coin_list(&mut self) -> Result<Vec<Coin>, Box<dyn std::error::Error>> {
        Ok(self.coin_gecko_client.get_coin_list().await?)
    }

    pub async fn update_prices_assets(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let coin_list = self.get_coin_list().await?;
        // assets
        let mut assets = coin_list
            .into_iter()
            .flat_map(|x| self.get_prices_assets_for_coin(x))
            .collect::<Vec<_>>();
        // native assets
        assets.extend(Chain::all().into_iter().map(|x| PriceAsset {
            asset_id: x.as_ref().to_string(),
            price_id: get_coingecko_market_id_for_chain(x).to_string(),
        }));
        self.price_client.set_prices_assets(assets)
    }

    pub async fn update_prices_all(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        self.update_prices(std::u32::MAX).await
    }

    pub async fn update_prices(&mut self, pages: u32) -> Result<usize, Box<dyn std::error::Error>> {
        let coin_markets = self
            .coin_gecko_client
            .get_all_coin_markets(250, pages)
            .await?;

        let prices = coin_markets
            .into_iter()
            .map(|market| price_for_market(market.clone()))
            .collect::<HashSet<Price>>()
            .into_iter()
            .collect::<Vec<Price>>();

        self.price_client.set_prices(prices)
    }

    pub async fn update_charts(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let prices = self.price_client.get_prices()?;
        let charts = prices
            .clone()
            .into_iter()
            .map(|x| x.as_chartcoin())
            .collect::<Vec<ChartCoinPrice>>();

        for chunk in charts.chunks(1000) {
            self.price_client.set_charts(chunk.to_vec()).await?;
        }

        Ok(charts.len())
    }

    pub fn get_prices_assets_for_coin(&mut self, coin: Coin) -> Vec<PriceAsset> {
        return coin
            .platforms
            .clone()
            .into_iter()
            .flat_map(|(platform, token_id)| {
                let platform = get_chain_for_coingecko_platform_id(platform.as_str());
                if let Some(chain) = platform {
                    let token_id = token_id.unwrap_or_default();
                    if !token_id.is_empty() {
                        if let Some(asset_id) = get_asset_id(chain, token_id) {
                            return Some(PriceAsset {
                                asset_id,
                                price_id: coin.id.clone(),
                            });
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>();
    }

    pub async fn update_charts_all(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let coin_list = self.get_coin_list().await?;

        for coin_id in coin_list.clone() {
            let prices = self
                .coin_gecko_client
                .get_market_chart(coin_id.id.as_str())
                .await;

            match prices {
                Ok(prices) => {
                    let charts = prices
                        .prices
                        .clone()
                        .into_iter()
                        .map(|x| ChartCoinPrice {
                            coin_id: coin_id.id.clone(),
                            price: x[1],
                            created_at: (x[0] as u64) / 1000,
                        })
                        .collect::<Vec<ChartCoinPrice>>();

                    match self.price_client.set_charts(charts).await {
                        Ok(_) => {
                            println!("set charts {}", coin_id.id.clone());
                        }
                        Err(err) => {
                            println!("set charts error: {}", err);
                        }
                    };

                    println!("update charts {}", coin_id.id.clone());

                    thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(err) => {
                    println!("update charts error: {}", err);
                    continue;
                }
            }
        }
        Ok(coin_list.len())
    }

    pub async fn update_prices_cache(&mut self) -> Result<usize, Box<dyn Error>> {
        let (prices_assets, prices, rates) = (
            self.price_client.get_prices_assets()?,
            self.price_client.get_prices()?,
            self.price_client.get_fiat_rates()?,
        );

        let prices_assets_map: HashMap<String, HashSet<String>> =
            prices_assets
                .into_iter()
                .fold(HashMap::new(), |mut map, price_asset| {
                    map.entry(price_asset.price_id.clone())
                        .or_default()
                        .insert(price_asset.asset_id);
                    map
                });

        let base_rate = rates
            .iter()
            .find(|x| x.symbol == DEFAULT_FIAT_CURRENCY)
            .map(|x| x.rate)
            .unwrap();

        for rate in rates.iter() {
            let prices = prices
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

            self.price_client
                .set_cache_prices(rate.symbol.as_str(), prices)
                .await?;
        }

        Ok(0)
    }

    pub async fn update_fiat_rates(&mut self) -> Result<usize, Box<dyn Error>> {
        let rates = self
            .coin_gecko_client
            .get_fiat_rates()
            .await?
            .into_iter()
            .map(FiatRate::from_primitive)
            .collect::<Vec<_>>();

        let count = self.price_client.set_fiat_rates(rates).await?;
        Ok(count)
    }

    pub async fn clean_outdated_assets(
        &mut self,
        seconds: u64,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let time = Utc::now() - Duration::seconds(seconds as i64);
        self.price_client
            .delete_prices_updated_at_before(time.naive_utc())
    }
}

fn price_for_market(market: CoinMarket) -> Price {
    Price::new(
        market.id,
        market.current_price.unwrap_or_default(),
        market.price_change_percentage_24h.unwrap_or_default(),
        market.market_cap.unwrap_or_default(),
        market.market_cap_rank.unwrap_or_default(),
        market.total_volume.unwrap_or_default(),
        market.circulating_supply.unwrap_or_default(),
        market.total_supply.unwrap_or_default(),
        market.max_supply.unwrap_or_default(),
        market.last_updated.map(|x| x.naive_local()),
    )
}

fn get_asset_id(chain: Chain, token_id: String) -> Option<String> {
    if token_id.is_empty() {
        return Some(chain.as_ref().to_string());
    }
    let token_id = AssetId::format_token_id(chain, token_id)?;
    format!("{}_{}", chain.as_ref(), token_id).into()
}
