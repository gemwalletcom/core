use coingecko::{get_chain_for_coingecko_platform_id, CoinGeckoClient, CoinInfo};
use primitives::{Asset, AssetDetails, AssetId, AssetLinks, AssetScore, AssetType};
use std::collections::HashSet;
use std::error::Error;
use storage::DatabaseClient;
pub struct AssetUpdater {
    coin_gecko_client: CoinGeckoClient,
    database: DatabaseClient,
}

impl AssetUpdater {
    pub fn new(coin_gecko_client: CoinGeckoClient, database_url: &str) -> Self {
        AssetUpdater {
            coin_gecko_client,
            database: DatabaseClient::new(database_url),
        }
    }

    pub async fn update_assets(&mut self) -> Result<usize, Box<dyn Error>> {
        let coin_list: Vec<_> = self
            .database
            .get_prices()?
            .into_iter()
            .map(|x| x.id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        for coin in coin_list.clone() {
            match self.coin_gecko_client.get_coin(coin.clone().as_str()).await {
                Ok(coin_info) => {
                    let result = self.get_assets_from_coin_info(coin_info);
                    for (asset, asset_score, asset_details) in result {
                        let _ = self.update_asset(asset, asset_score, asset_details).await;
                    }
                }
                Err(err) => {
                    println!("error getting coin info for coin {}: {}", coin.clone(), err);
                }
            }
        }
        Ok(coin_list.len())
    }

    fn get_assets_from_coin_info(
        &self,
        coin_info: CoinInfo,
    ) -> Vec<(Asset, AssetScore, AssetDetails)> {
        let asset_details = self.get_asset_details(coin_info.clone());

        let values = coin_info
            .clone()
            .detail_platforms
            .into_iter()
            .filter_map(|(coin_id, detail_platform)| {
                let chain = get_chain_for_coingecko_platform_id(coin_id.as_str());
                if let (Some(chain), Some(detail_platform)) = (chain, detail_platform) {
                    return Some((chain, Some(detail_platform)));
                }
                None
            })
            .collect::<Vec<_>>();

        values
            .into_iter()
            .flat_map(|(chain, platform)| {
                if let (Some(asset_type), Some(platform)) =
                    (chain.default_asset_type(), platform.clone())
                {
                    if platform.contract_address.is_empty() || platform.decimal_place.is_none() {
                        return None;
                    }
                    let token_id = AssetId::format_token_id(chain, platform.contract_address)?;
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
                } else {
                    None
                }
            })
            .map(|x| {
                (
                    x.clone(),
                    self.get_asset_score(x.clone(), coin_info.clone()),
                    asset_details.clone(),
                )
            })
            .collect::<Vec<_>>()
    }

    fn get_asset_score(&self, asset: Asset, coin_info: CoinInfo) -> AssetScore {
        if asset.asset_type == AssetType::NATIVE {
            return AssetScore {
                rank: asset.chain().rank(),
            };
        }
        let mut rank = 12;

        // market cap calculation
        let market_cap_rank = coin_info.market_cap_rank.unwrap_or_default();
        if market_cap_rank > 0 && market_cap_rank < 100 {
            rank += 4;
        } else if market_cap_rank < 500 {
            rank += 3;
        } else if market_cap_rank < 1000 {
            rank += 2;
        } else if market_cap_rank < 2000 {
            rank += 1;
        }

        if coin_info.platforms.len() > 6 {
            rank += 2;
        } else if coin_info.platforms.len() > 3 {
            rank += 1;
        }

        // social
        if let Some(data) = coin_info.community_data {
            let twitter_followers = data.twitter_followers.unwrap_or_default();
            if twitter_followers > 128_000 {
                rank += 1;
            }
        }
        let watchlist = coin_info.watchlist_portfolio_users.unwrap_or_default() as i32;
        if watchlist > 1_000_000 {
            rank += 2;
        } else if watchlist > 250_000 {
            rank += 1;
        }

        rank += asset.chain().rank() / 20;

        AssetScore { rank }
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
                .filter(|x| !x.clone().unwrap_or("".to_string()).is_empty())
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
            explorer: explorer.unwrap_or_default(),
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

    // asset, asset details
    pub async fn update_asset(
        &mut self,
        asset: Asset,
        asset_score: AssetScore,
        asset_details: AssetDetails,
    ) -> Result<(), Box<dyn Error>> {
        let details = storage::models::asset::AssetDetail::from_primitive(
            asset.id.to_string().as_str(),
            asset_details,
        );
        let asset = storage::models::asset::Asset::from_primitive(asset);
        let asset_id = asset.id.as_str();
        let _ = self.database.add_assets(vec![asset.clone()]);
        let _ = self.database.add_assets_details(vec![details]);
        let _ = self.database.update_asset_rank(asset_id, asset_score.rank);
        Ok(())
    }
}
