use crate::client::Client;
use crate::coingecko::{CoinGeckoClient, CoinInfo, CoinMarket};
use crate::price_mapper::get_chain_for_coingecko_id;
use crate::DEFAULT_FIAT_CURRENCY;
use primitives::chain::Chain;
use primitives::{Asset, AssetDetails, AssetId, AssetLinks, EthereumAddress};
use std::collections::HashSet;
use std::error::Error;
use storage::models::{Chart, Price};

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

    pub async fn update_prices(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let coin_list = self.coin_gecko_client.get_coin_list().await?;
        let coins_map = CoinGeckoClient::convert_coin_vec_to_map(coin_list.clone());
        let coin_markets = self.coin_gecko_client.get_all_coin_markets(250, 10).await?;

        // currently using as a map, until fix duplicated values in the vector.
        let mut prices_map: HashSet<Price> = HashSet::new();

        for market in coin_markets {
            let chain = get_chain_for_coingecko_id(market.id.as_str());

            match chain {
                Some(value) => {
                    if let Some(asset_id) = get_asset_id(value, "".to_string()) {
                        prices_map.insert(asset_price_map(asset_id, market.clone()));
                    }
                    // special case.
                    if value.as_str() == Chain::Binance.as_str() {
                        prices_map.insert(asset_price_map(
                            Chain::SmartChain.as_str().to_string(),
                            market.clone(),
                        ));
                        prices_map.insert(asset_price_map(
                            Chain::OpBNB.as_str().to_string(),
                            market.clone(),
                        ));
                    }
                    if value.as_str() == Chain::Ethereum.as_str() {
                        prices_map.insert(asset_price_map(
                            Chain::Arbitrum.as_str().to_string(),
                            market.clone(),
                        ));
                        prices_map.insert(asset_price_map(
                            Chain::Optimism.as_str().to_string(),
                            market.clone(),
                        ));
                        prices_map.insert(asset_price_map(
                            Chain::Base.as_str().to_string(),
                            market.clone(),
                        ));
                    }
                }
                None => {
                    let coin_map = coins_map.get(market.id.as_str()).unwrap();
                    for (platform, token_id) in coin_map.platforms.clone().into_iter() {
                        let platform = get_chain_for_coingecko_id(platform.as_str());
                        if let Some(value) = platform {
                            let token_id = token_id.unwrap_or_default();
                            if !token_id.is_empty() {
                                if let Some(asset_id) = get_asset_id(value, token_id) {
                                    prices_map.insert(asset_price_map(asset_id, market.clone()));
                                }
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
            self.price_client
                .set_cache_prices(rate.symbol.as_str(), rate_prices)
                .await?;
        }
        Ok(prices.len())
    }

    pub async fn update_fiat_rates(&mut self) -> Result<usize, Box<dyn Error>> {
        let rates = self.coin_gecko_client.get_fiat_rates().await?;
        let count = self.price_client.set_fiat_rates(rates).await?;
        Ok(count)
    }

    pub async fn update_assets(&mut self) -> Result<usize, Box<dyn Error>> {
        let coin_list = self
            .coin_gecko_client
            .get_coin_markets(0, 250)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect::<Vec<_>>();

        for coin in coin_list.clone() {
            let coin_info = self.coin_gecko_client.get_coin(coin.as_str()).await?;

            if coin_info.preview_listing || coin_info.market_cap_rank.unwrap_or(999999) > 250 {
                //println!("early exit loop for {}", coin_info.id);
                continue;
            }
            let result = self.get_assets_from_coin_info(coin_info);
            for (asset, asset_details) in result {
                self.price_client.update_asset(asset, asset_details).await?;
            }
        }

        Ok(coin_list.len())
    }

    fn get_assets_from_coin_info(&self, coin_info: CoinInfo) -> Vec<(Asset, AssetDetails)> {
        let asset_details = self.get_asset_details(coin_info.clone());

        let mut values = coin_info
            .clone()
            .detail_platforms
            .into_iter()
            .filter_map(|x| {
                if let Some(chain) = get_chain_for_coingecko_id(x.0.as_str()) {
                    return Some((chain, Some(x.1.unwrap())));
                }
                None
            })
            .collect::<Vec<_>>();

        if let Some(chain) = get_chain_for_coingecko_id(coin_info.clone().id.as_str()) {
            values.push((chain, None));
        }

        values
            .into_iter()
            .flat_map(|(chain, platform)| {
                if let (Some(asset_type), Some(platform)) =
                    (chain.default_asset_type(), platform.clone())
                {
                    if platform.contract_address.is_empty() || platform.decimal_place.is_none() {
                        return None;
                    }
                    let token_id = format_token_id(chain, platform.contract_address)?;
                    let decimals = platform.decimal_place.unwrap_or_default();
                    let asset_id = AssetId {
                        chain,
                        token_id: token_id.into(),
                    };
                    let asset = Asset {
                        id: asset_id,
                        name: coin_info.clone().name,
                        symbol: coin_info.clone().symbol.to_uppercase(),
                        decimals,
                        asset_type,
                    };
                    Some(asset)
                } else if platform.is_none() {
                    return Some(Asset::from_chain(chain));
                } else {
                    None
                }
            })
            .map(|x| (x, asset_details.clone()))
            .collect::<Vec<_>>()
    }

    fn get_asset_details(&self, coin_info: CoinInfo) -> AssetDetails {
        let links = coin_info.links.clone();
        let homepage = links
            .clone()
            .homepage
            .into_iter()
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>()
            .first()
            .cloned();
        let explorer = if coin_info.asset_platform_id.is_none() {
            links
                .clone()
                .blockchain_site
                .into_iter()
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>()
                .first()
                .cloned()
        } else {
            None
        };
        let twitter = if links
            .clone()
            .twitter_screen_name
            .unwrap_or_default()
            .is_empty()
        {
            None
        } else {
            Some(format!(
                "https://x.com/{}",
                links.clone().twitter_screen_name.unwrap_or_default()
            ))
        };
        let facebook = if links
            .clone()
            .facebook_username
            .unwrap_or_default()
            .is_empty()
        {
            None
        } else {
            Some(format!(
                "https://facebook.com/{}",
                links.clone().facebook_username.unwrap_or_default()
            ))
        };
        let telegram = if links
            .clone()
            .telegram_channel_identifier
            .unwrap_or_default()
            .is_empty()
        {
            None
        } else {
            Some(format!(
                "https://t.me/{}",
                links
                    .clone()
                    .telegram_channel_identifier
                    .unwrap_or_default()
            ))
        };
        let reddit = if links.clone().subreddit_url.unwrap_or_default() == "https://www.reddit.com"
        {
            None
        } else {
            links.clone().subreddit_url
        };
        let coingecko = format!(
            "https://www.coingecko.com/coins/{}",
            coin_info.id.to_lowercase()
        );
        let coinmarketcap = format!(
            "https://coinmarketcap.com/currencies/{}",
            coin_info.id.to_lowercase()
        );
        let discord = links
            .clone()
            .chat_url
            .into_iter()
            .filter(|x| x.contains("discord.com"))
            .collect::<Vec<_>>()
            .first()
            .cloned();
        let repos = links
            .clone()
            .repos_url
            .get("github")
            .cloned()
            .unwrap_or_default();
        let github = repos
            .into_iter()
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>()
            .first()
            .cloned();

        let links = AssetLinks {
            homepage,
            explorer,
            twitter,
            telegram,
            github,
            youtube: None,
            facebook,
            reddit,
            coingecko: Some(coingecko),
            coinmarketcap: Some(coinmarketcap),
            discord,
        };

        AssetDetails::from_links(links)
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
        return Some(chain.as_str().to_string());
    }
    let token_id = format_token_id(chain, token_id)?;
    format!("{}_{}", chain.as_str(), token_id).into()
}

fn format_token_id(chain: Chain, token_id: String) -> Option<String> {
    match chain {
        Chain::Ethereum
        | Chain::SmartChain
        | Chain::Polygon
        | Chain::Arbitrum
        | Chain::Optimism
        | Chain::Base
        | Chain::AvalancheC
        | Chain::OpBNB
        | Chain::Fantom
        | Chain::Gnosis => Some(EthereumAddress::parse(&token_id)?.to_checksum()),
        Chain::Bitcoin
        | Chain::Litecoin
        | Chain::Binance
        | Chain::Solana
        | Chain::Thorchain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Ton
        | Chain::Tron
        | Chain::Doge
        | Chain::Aptos
        | Chain::Sui
        | Chain::Ripple => None,
    }
}
