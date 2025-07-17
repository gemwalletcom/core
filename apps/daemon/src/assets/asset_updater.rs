use chain_primitives::format_token_id;
use coingecko::mapper::COINGECKO_CHAIN_MAP;
use coingecko::{get_chain_for_coingecko_platform_id, CoinGeckoClient, CoinInfo};
use primitives::{Asset, AssetBasic, AssetId, AssetLink, AssetProperties, AssetScore, AssetType, LinkType};
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

    pub async fn update_assets(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = self
            .database
            .prices()
            .get_prices()?
            .into_iter()
            .map(|x| x.id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<String>>();
        self.update_assets_ids(ids).await
    }

    pub async fn update_trending_assets(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = self.coin_gecko_client.get_search_trending().await?.get_coins_ids();
        self.update_assets_ids(ids).await
    }

    pub async fn update_recently_added_assets(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = self.coin_gecko_client.get_coin_list_new().await?.ids().iter().take(10).cloned().collect();
        self.update_assets_ids(ids).await
    }

    async fn update_assets_ids(&mut self, ids: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        for coin in ids.clone() {
            match self.coin_gecko_client.get_coin(&coin).await {
                Ok(coin_info) => {
                    let result = self.get_assets_from_coin_info(coin_info.clone());

                    let asset_links = self.get_asset_links(coin_info.clone());

                    if result.is_empty() {
                        if let Some(chain) = COINGECKO_CHAIN_MAP.get(&coin) {
                            let _ = self.update_links(&chain.as_asset_id().to_string(), asset_links).await;
                        }
                    } else {
                        for (asset, asset_score) in result.clone() {
                            let _ = self.update_asset(asset, asset_score, asset_links.clone()).await;
                        }
                    }
                }
                Err(err) => {
                    println!("error getting coin info for coin {coin}: {err}");
                }
            }
        }
        Ok(ids.len())
    }

    fn get_assets_from_coin_info(&self, coin_info: CoinInfo) -> Vec<(Asset, AssetScore)> {
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
                if let (Some(asset_type), Some(platform)) = (chain.default_asset_type(), platform.clone()) {
                    if platform.contract_address.is_empty() || platform.decimal_place.is_none() {
                        return None;
                    }
                    let token_id = format_token_id(chain, platform.contract_address)?;
                    let decimals = platform.decimal_place.unwrap_or_default();
                    let asset_id = AssetId {
                        chain,
                        token_id: token_id.into(),
                    };
                    let asset = Asset::new(asset_id, coin_info.clone().name, coin_info.clone().symbol.to_uppercase(), decimals, asset_type);
                    Some(asset)
                } else {
                    None
                }
            })
            .map(|x| (x.clone(), self.get_asset_score(x.clone(), coin_info.clone())))
            .collect::<Vec<_>>()
    }

    fn get_asset_score(&self, asset: Asset, coin_info: CoinInfo) -> AssetScore {
        if asset.asset_type == AssetType::NATIVE {
            return AssetScore::new(asset.chain().rank());
        }
        let mut rank = 12;

        // market cap calculation
        let market_cap_rank = coin_info.market_cap_rank.unwrap_or_default();
        rank += match market_cap_rank {
            1..25 => 15,
            25..50 => 12,
            50..100 => 10,
            100..250 => 8,
            250..500 => 6,
            500..1000 => 4,
            1000..2000 => 2,
            2000..4000 => 0,
            4000..5000 => -1,
            _ => -2, // Default case (no change)
        };

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

        rank = rank.max(16);

        AssetScore::new(rank)
    }

    fn get_asset_links(&self, coin_info: CoinInfo) -> Vec<AssetLink> {
        let links = coin_info.links.clone();

        let mut results = vec![];

        if let Some(value) = links.clone().twitter_screen_name {
            if !value.is_empty() {
                results.push(AssetLink::new(&format!("https://x.com/{value}"), LinkType::X));
            }
        }

        if let Some(value) = links
            .clone()
            .homepage
            .into_iter()
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>()
            .first()
            .cloned()
        {
            let exclude_domains = ["https://t.me"];
            if !value.is_empty() && !exclude_domains.iter().any(|&domain| value.contains(domain)) {
                results.push(AssetLink::new(&value, LinkType::Website));
            }
        }

        if let Some(value) = links.clone().telegram_channel_identifier {
            if !value.is_empty() {
                results.push(AssetLink::new(&format!("https://t.me/{value}"), LinkType::Telegram));
            }
        };

        results.push(AssetLink::new(
            &format!("https://www.coingecko.com/coins/{}", coin_info.id.to_lowercase()),
            LinkType::Coingecko,
        ));

        if let Some(value) = links
            .clone()
            .chat_url
            .into_iter()
            .filter(|x| x.contains("discord.com"))
            .collect::<Vec<_>>()
            .first()
            .cloned()
        {
            results.push(AssetLink::new(&value, LinkType::Discord));
        };

        if let Some(value) = links
            .clone()
            .repos_url
            .get("github")
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>()
            .first()
            .cloned()
        {
            results.push(AssetLink::new(&value, LinkType::GitHub));
        };

        results
    }

    // asset, asset details
    pub async fn update_asset(&mut self, asset: Asset, score: AssetScore, asset_links: Vec<AssetLink>) -> Result<(), Box<dyn Error>> {
        let properties = AssetProperties::default(asset.id.clone());
        let asset_id = asset.id.to_string();
        let asset = AssetBasic::new(asset.clone(), properties, score);
        let _ = self.database.assets().add_assets(vec![asset]);
        let _ = self.update_links(&asset_id, asset_links.clone()).await;
        Ok(())
    }

    pub async fn update_links(&mut self, asset_id: &str, asset_links: Vec<AssetLink>) -> Result<(), Box<dyn Error>> {
        let _ = self.database.assets_links().add_assets_links(asset_id, asset_links);
        Ok(())
    }
}
