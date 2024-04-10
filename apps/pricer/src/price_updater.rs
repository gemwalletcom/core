use crate::client::PriceClient;
use crate::DEFAULT_FIAT_CURRENCY;
use chrono::{Duration, Utc};
use coingecko::mapper::{get_chain_for_coingecko_platform_id, get_coingecko_market_id_for_chain};
use coingecko::{Coin, CoinGeckoClient, CoinMarket};
use primitives::chain::Chain;
use primitives::AssetId;
use std::collections::HashSet;
use std::error::Error;
use std::thread;
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
        match self.price_client.get_coingecko_coins_list().await {
            Ok(value) => {
                let coin_list = serde_json::from_str::<Vec<Coin>>(&value)?;
                Ok(coin_list)
            }
            Err(_) => {
                let coin_list = self.coin_gecko_client.get_coin_list().await?;
                let string = serde_json::to_string(&coin_list)?;
                self.price_client.set_coingecko_coins_list(string).await?;
                Ok(coin_list)
            }
        }
    }

    pub async fn update_prices(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let coin_list = self.get_coin_list().await?;
        let coins_map = CoinGeckoClient::convert_coin_vec_to_map(coin_list.clone());
        let coin_markets = self.coin_gecko_client.get_all_coin_markets(250, 1).await?;
        //TODO: currently using as a map, until fix duplicated values in the vector.
        let mut prices_map: HashSet<Price> = HashSet::new();

        for chain in Chain::all() {
            let market_id = get_coingecko_market_id_for_chain(chain);
            if let Some(market) = coin_markets.iter().find(|x| x.id == market_id) {
                prices_map.insert(asset_price_map(chain.as_ref().to_string(), market.clone()));
            }
        }
        println!("Chain::all(): {:?}", Chain::all().len());
        println!("prices_map: {:?}", prices_map);

        // for market in coin_markets {
        //     let coin_map = coins_map.get(market.id.as_str()).unwrap();
        //     let prices = self.get_prices_for_coin_market(coin_map.clone(), market.clone());
        //     prices_map.extend(prices);
        // }

        let prices: Vec<Price> = prices_map.into_iter().collect();

        println!("prices: {:?}", prices);

        let count = self.price_client.set_prices(prices.clone())?;

        let charts = prices
            .clone()
            .into_iter()
            .map(|x| ChartCoinPrice {
                coin_id: x.coin_id,
                price: x.price,
                created_at: x.last_updated_at.timestamp() as u64,
            })
            .collect();

        let _ = self.price_client.set_charts(charts).await?;

        Ok(count)
    }

    pub async fn update_price(
        &mut self,
        id: &str,
    ) -> Result<Vec<Price>, Box<dyn std::error::Error>> {
        let market = self.coin_gecko_client.get_coin_markets_id(id).await?;
        let coin_list = self.get_coin_list().await?;
        let coins_map = CoinGeckoClient::convert_coin_vec_to_map(coin_list.clone());
        let coin = coins_map.get(market.id.as_str()).unwrap();
        let prices = self.get_prices_for_coin_market(coin.clone(), market.clone());
        Ok(prices)
    }

    pub fn get_prices_for_coin_market(&mut self, coin: Coin, market: CoinMarket) -> Vec<Price> {
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
                            let price = asset_price_map(asset_id, market.clone());
                            return Some(price);
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>();
    }

    pub async fn update_charts(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
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

    pub async fn update_cache(&mut self) -> Result<usize, Box<dyn Error>> {
        let prices = self.price_client.get_prices()?;
        let rates = self.price_client.get_fiat_rates()?;
        let base_rate = rates
            .iter()
            .find(|x| x.symbol == DEFAULT_FIAT_CURRENCY)
            .map(|x| x.rate)
            .unwrap();

        for rate in rates.iter() {
            let mut rate_prices: Vec<Price> = vec![];
            for price in &prices {
                let mut new_price = price.clone();
                let rate_multiplier = rate.rate / base_rate;
                new_price.price = price.price * rate_multiplier;
                new_price.market_cap = price.market_cap * rate_multiplier;
                new_price.total_volume = price.total_volume * rate_multiplier;
                rate_prices.push(new_price)
            }
            self.price_client
                .set_cache_prices(rate.symbol.as_str(), rate_prices)
                .await?;
        }
        Ok(prices.len())
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

fn asset_price_map(asset_id: String, market: CoinMarket) -> Price {
    Price::new(
        asset_id,
        market.id,
        market.current_price.unwrap_or_default(),
        market.price_change_percentage_24h.unwrap_or_default(),
        market.market_cap.unwrap_or_default(),
        market.market_cap_rank.unwrap_or_default(),
        market.total_volume.unwrap_or_default(),
        market.circulating_supply.unwrap_or_default(),
        market.total_supply.unwrap_or_default(),
        market.max_supply.unwrap_or_default(),
    )
}

fn get_asset_id(chain: Chain, token_id: String) -> Option<String> {
    if token_id.is_empty() {
        return Some(chain.as_ref().to_string());
    }
    let token_id = AssetId::format_token_id(chain, token_id)?;
    format!("{}_{}", chain.as_ref(), token_id).into()
}
