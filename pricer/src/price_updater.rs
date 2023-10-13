use crate::DEFAULT_FIAT_CURRENCY;
use crate::coingecko::{CoinGeckoClient, CoinMarket, CoinInfo};
use crate::client:: Client;
use crate::price_mapper::get_chain_for_coingecko_id;
use primitives::{Asset, AssetId};
use primitives::chain::Chain;
use storage::models::{Price, Chart};
use std::collections::HashSet;
use std::error::Error;
use std::str::FromStr;

pub struct PriceUpdater {
    coin_gecko_client: CoinGeckoClient,
    price_client: Client,
}

impl PriceUpdater {
    pub fn new(price_client: Client, coin_gecko_client: CoinGeckoClient) -> Self {
        PriceUpdater {
            coin_gecko_client,
            price_client,
        }
    }

    pub async fn update_prices(&mut self) ->  Result<usize, Box<dyn std::error::Error>>  {
        let coin_list = self.coin_gecko_client.get_coin_list().await?;
        let coins_map = CoinGeckoClient::convert_coin_vec_to_map(coin_list.clone());
        let coin_markets = self.coin_gecko_client.get_all_coin_markets(250, 10).await?;

        //println!("coin_list: {}", coin_list.len());
        //println!("coin_markets: {}", coin_markets.len());

        // currently using as a map, until fix duplicated values in the vector.
        let mut prices_map: HashSet<Price> = HashSet::new();
        
        for market in coin_markets {
            
            let chain = get_chain_for_coingecko_id(market.id.as_str());
            
            match chain {
                Some(value) => {
                    let asset_id = get_asset_id(value, "".to_string());
                    prices_map.insert(
                        asset_price_map(asset_id, market.clone())
                    );
                    // special case.
                    if value.as_str() == Chain::Binance.as_str() {
                        prices_map.insert(
                            asset_price_map(Chain::SmartChain.as_str().to_string(), market.clone())
                        );
                        prices_map.insert(
                            asset_price_map(Chain::OpBNB.as_str().to_string(), market.clone())
                        );
                    }
                    if value.as_str() == Chain::Ethereum.as_str() {
                        prices_map.insert(
                            asset_price_map(Chain::Arbitrum.as_str().to_string(), market.clone())
                        );
                        prices_map.insert(
                            asset_price_map(Chain::Optimism.as_str().to_string(), market.clone())
                        );
                        prices_map.insert(
                            asset_price_map(Chain::Base.as_str().to_string(), market.clone())
                        );
                    }
                }
                None=> {
                    let coin_map = coins_map.get(market.id.as_str()).unwrap();
                    for (platform, token_id) in coin_map.platforms.clone().into_iter() {
                        let platform = get_chain_for_coingecko_id(platform.as_str());
                        if let Some(value) = platform {
                            let token_id = token_id.unwrap_or_default();
                            if !token_id.is_empty() {
                                let asset_id = get_asset_id(value, token_id);
                                prices_map.insert(
                                    asset_price_map(asset_id, market.clone())
                                );
                            }
                        }
                    }
                }
            }
        }
        let prices: Vec<Price> = prices_map.into_iter().collect();
        let count = self.price_client.set_prices(prices.clone()).await?;

        let mut charts_map: HashSet<Chart> = HashSet::new();
        for price in prices.clone() {
            charts_map.insert(price.chart_value());
        }
        let charts = charts_map.into_iter().collect();

        let _ = self.price_client.set_charts(charts).await?;

        Ok(count)
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
            self.price_client.set_cache_prices(rate.symbol.as_str(), rate_prices).await?;
        }
        Ok(prices.len())
    }

    pub async fn update_fiat_rates(&mut self) -> Result<usize, Box<dyn Error>> {
        let rates = self.coin_gecko_client.get_fiat_rates().await?;
        let count = self.price_client.set_fiat_rates(rates).await?;
        Ok(count)
    }

    pub async fn update_assets(&mut self) -> Result<usize, Box<dyn Error>> {
        let coin_list = self.coin_gecko_client.get_coin_list().await?;
        
        for coin in coin_list {
            let chains = vec![Chain::Ethereum];
            let platforms = coin.platforms.into_iter().flat_map(|x| {
                get_chain_for_coingecko_id(x.0.as_str())
            })
            .filter(|x| chains.contains(x))
            .collect::<Vec<_>>();

            if platforms.is_empty(){
                continue;
            }
            println!("coin_info: {}", coin.id);
            let coin_info = self.coin_gecko_client.get_coin(coin.id.as_str()).await?;
            
            if coin_info.preview_listing || coin_info.market_cap_rank.unwrap_or(999999) > 100 {
                //println!("early exit loop for {}", coin_info.id);
                continue;
            }
            let assets = self.get_assets_from_coin_info(coin_info);

            println!("assets: {:?}, coin_id: {} \n", assets, coin.id);
        }

        Ok(0)
    }

    fn get_assets_from_coin_info(&self, coin_info: CoinInfo) -> Vec<Asset> {
        return coin_info.detail_platforms.into_iter().filter_map(|x| {
            if let (Some(chain), Some(platform)) = (get_chain_for_coingecko_id(x.0.as_str()), x.1) {
                return Some((chain, platform))
            }
            return None
        })
        .flat_map(|(chain, platform)| {
            let token_id = format_token_id(chain, platform.contract_address);
            let decimals = platform.decimal_place.unwrap_or_default();
            let asset_id = AssetId{chain, token_id: token_id.into()};
            let asset = Asset{
                id: asset_id,
                name: coin_info.name.clone(),
                symbol: coin_info.symbol.to_uppercase().clone(),
                decimals,
                asset_type: chain.default_asset_type().unwrap(),
            };
            return Some(asset)
        })
        .collect::<Vec<_>>();
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
    )
}

fn get_asset_id(chain: Chain, token_id: String) -> String {
    if token_id.is_empty() {
        return chain.as_str().to_string()
    }
    format!("{}_{}", chain.as_str(), format_token_id(chain, token_id))
}

fn format_token_id(chain: Chain, token_id: String) -> String {
    match chain {
        Chain::Ethereum |
        Chain::SmartChain |
        Chain::Polygon |
        Chain::Arbitrum |
        Chain::Optimism => {
            return ethaddr::Address::from_str(token_id.as_str()).unwrap().to_string();
        }
        _ => {
            token_id
        }
    }
}