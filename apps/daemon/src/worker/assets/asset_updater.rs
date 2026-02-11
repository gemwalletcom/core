use cacher::{CacheKey, CacherClient};
use chain_primitives::format_token_id;
use coingecko::{COINGECKO_CHAIN_MAP, CoinGeckoClient, CoinInfo, get_chain_for_coingecko_platform_id, get_coingecko_market_id_for_chain};
use primitives::{Asset, AssetBasic, AssetId, AssetLink, AssetProperties, AssetScore, AssetType, Chain, LinkType};
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;
use storage::models::price::{NewPriceRow, PriceAssetRow};
use storage::{AssetUpdate, AssetsLinksRepository, AssetsRepository, Database, PricesRepository};
use streamer::{StreamProducer, StreamProducerQueue};

pub struct AssetProcessor {
    coin_gecko_client: CoinGeckoClient,
    database: Database,
    cacher: CacherClient,
}

impl AssetProcessor {
    pub fn new(coin_gecko_client: CoinGeckoClient, database: Database, cacher: CacherClient) -> Self {
        Self {
            coin_gecko_client,
            database,
            cacher,
        }
    }

    pub async fn process_coin_update(&self, coin_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let coin_info = self.get_coin_info_cached(coin_id).await?;
        let assets = Self::map_assets(&coin_info);
        let links = Self::map_links(&coin_info);

        self.store_prices(coin_id, &assets)?;
        self.store_assets(&assets, &links, coin_id)?;

        Ok(1)
    }

    // MARK: - Data Fetching

    async fn get_coin_info_cached(&self, coin_id: &str) -> Result<CoinInfo, Box<dyn Error + Send + Sync>> {
        let coin_id_owned = coin_id.to_string();
        let client = self.coin_gecko_client.clone();

        self.cacher
            .get_or_set_cached(CacheKey::PricerCoinInfo(coin_id), || async move { client.get_coin(&coin_id_owned).await })
            .await
    }

    // MARK: - Storage

    fn store_prices(&self, coin_id: &str, assets: &[(Asset, AssetScore)]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.database.prices()?.add_prices(vec![NewPriceRow::new(coin_id.to_string())])?;

        let price_assets = assets
            .iter()
            .map(|(asset, _)| PriceAssetRow::new(asset.id.to_string(), coin_id.to_string()))
            .collect::<Vec<_>>();

        self.database.prices()?.set_prices_assets(price_assets)?;
        Ok(())
    }

    fn store_assets(&self, assets: &[(Asset, AssetScore)], links: &[AssetLink], coin_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        if assets.is_empty() {
            if let Some(chain) = COINGECKO_CHAIN_MAP.get(coin_id) {
                self.store_links(&chain.as_asset_id().to_string(), links)?;
            }
        } else {
            for (asset, score) in assets {
                self.store_asset(asset, score, links)?;
            }
        }
        Ok(())
    }

    fn store_asset(&self, asset: &Asset, score: &AssetScore, links: &[AssetLink]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let properties = AssetProperties::default(asset.id.clone());
        let asset_id = asset.id.to_string();
        let asset_basic = AssetBasic::new(asset.clone(), properties, score.clone());
        self.database.assets()?.add_assets(vec![asset_basic])?;
        self.database.assets()?.update_assets(vec![asset_id.clone()], vec![AssetUpdate::Rank(score.rank)])?;
        self.store_links(&asset_id, links)?;
        Ok(())
    }

    fn store_links(&self, asset_id: &str, links: &[AssetLink]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.database.assets_links()?.add_assets_links(asset_id, links.to_vec())?;
        Ok(())
    }

    // MARK: - Mappers

    fn map_assets(coin_info: &CoinInfo) -> Vec<(Asset, AssetScore)> {
        coin_info
            .detail_platforms
            .iter()
            .filter_map(|(platform_id, detail)| {
                let chain = get_chain_for_coingecko_platform_id(platform_id)?;
                Some((chain, detail.as_ref()?))
            })
            .filter_map(|(chain, platform)| {
                let asset_type = chain.default_asset_type()?;
                let contract_address = platform.contract_address.as_ref().filter(|a| !a.is_empty())?;
                let decimals = platform.decimal_place?;
                let token_id = format_token_id(chain, contract_address.clone())?;
                let asset_id = AssetId { chain, token_id: token_id.into() };
                let asset = Asset::new(asset_id, coin_info.name.clone(), coin_info.symbol.to_uppercase(), decimals, asset_type);
                let score = Self::compute_score(&asset, coin_info);
                Some((asset, score))
            })
            .collect()
    }

    fn compute_score(asset: &Asset, coin_info: &CoinInfo) -> AssetScore {
        if asset.asset_type == AssetType::NATIVE {
            return AssetScore::new(asset.chain().rank());
        }

        let mut rank = 12;
        rank += Self::market_cap_rank_score(coin_info.market_cap_rank.unwrap_or_default());
        rank += Self::platform_diversity_score(coin_info.platforms.len());
        rank += Self::social_score(coin_info);
        rank += asset.chain().rank() / 20;
        rank = rank.max(16);

        AssetScore::new(rank)
    }

    fn market_cap_rank_score(market_cap_rank: i32) -> i32 {
        match market_cap_rank {
            1..25 => 15,
            25..50 => 12,
            50..100 => 10,
            100..250 => 8,
            250..500 => 6,
            500..1000 => 4,
            1000..2000 => 2,
            2000..4000 => 0,
            4000..5000 => -1,
            _ => -2,
        }
    }

    fn platform_diversity_score(platform_count: usize) -> i32 {
        if platform_count > 6 {
            2
        } else if platform_count > 3 {
            1
        } else {
            0
        }
    }

    fn social_score(coin_info: &CoinInfo) -> i32 {
        let twitter_score = coin_info
            .community_data
            .as_ref()
            .filter(|d| d.twitter_followers.unwrap_or_default() > 128_000)
            .map(|_| 1)
            .unwrap_or_default();

        let watchlist = coin_info.watchlist_portfolio_users.unwrap_or_default() as i32;
        let watchlist_score = if watchlist > 1_000_000 {
            2
        } else if watchlist > 250_000 {
            1
        } else {
            0
        };

        twitter_score + watchlist_score
    }

    fn map_links(coin_info: &CoinInfo) -> Vec<AssetLink> {
        let links = &coin_info.links;
        let mut results = vec![AssetLink::new(
            &format!("https://www.coingecko.com/coins/{}", coin_info.id.to_lowercase()),
            LinkType::Coingecko,
        )];

        if let Some(value) = links.twitter_screen_name.as_ref().filter(|v| !v.is_empty()) {
            results.push(AssetLink::new(&format!("https://x.com/{value}"), LinkType::X));
        }

        if let Some(value) = links.homepage.iter().find(|x| !x.is_empty()).filter(|v| !v.starts_with("https://t.me")) {
            results.push(AssetLink::new(value, LinkType::Website));
        }

        if let Some(value) = links.telegram_channel_identifier.as_ref().filter(|v| !v.is_empty()) {
            results.push(AssetLink::new(&format!("https://t.me/{value}"), LinkType::Telegram));
        }

        if let Some(value) = links.chat_url.iter().find(|x| x.contains("discord.com")) {
            results.push(AssetLink::new(value, LinkType::Discord));
        }

        if let Some(value) = links.repos_url.get("github").and_then(|urls| urls.iter().find(|x| !x.is_empty())) {
            results.push(AssetLink::new(value, LinkType::GitHub));
        }

        results
    }
}

#[derive(Clone, Copy)]
pub struct AssetUpdaterConfig {
    pub new_interval: Duration,
    pub existing_interval: Duration,
}

pub struct AssetUpdater {
    processor: AssetProcessor,
    stream_producer: StreamProducer,
    config: AssetUpdaterConfig,
}

impl AssetUpdater {
    pub fn new(coin_gecko_client: CoinGeckoClient, database: Database, cacher: CacherClient, stream_producer: StreamProducer, config: AssetUpdaterConfig) -> Self {
        Self {
            processor: AssetProcessor::new(coin_gecko_client, database, cacher),
            stream_producer,
            config,
        }
    }

    pub async fn update_existing_assets(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = self
            .processor
            .database
            .prices()?
            .get_prices()?
            .into_iter()
            .map(|x| x.id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<String>>();

        self.publish_updates(ids, self.config.existing_interval).await
    }

    pub async fn update_assets(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let assets = self.processor.coin_gecko_client.get_all_coin_markets(None, 250, 20).await?;
        let ids = assets.iter().map(|x| x.id.clone()).collect();
        self.publish_updates(ids, self.config.new_interval).await
    }

    pub async fn update_native_prices_assets(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let native_assets = Chain::all()
            .into_iter()
            .map(|x| PriceAssetRow::new(x.as_ref().to_string(), get_coingecko_market_id_for_chain(x).to_string()))
            .collect::<Vec<_>>();

        let ids = native_assets.iter().map(|x| x.price_id.clone()).collect();
        let _ = self.publish_updates(ids, self.config.new_interval).await;

        Ok(self.processor.database.prices()?.set_prices_assets(native_assets)?)
    }

    pub async fn update_trending_assets(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = self.processor.coin_gecko_client.get_search_trending().await?.get_coins_ids();
        self.publish_updates(ids, self.config.new_interval).await
    }

    pub async fn update_recently_added_assets(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = self.processor.coin_gecko_client.get_coin_list_new().await?.ids().iter().take(10).cloned().collect();
        self.publish_updates(ids, self.config.new_interval).await
    }

    async fn publish_updates(&self, ids: Vec<String>, interval: Duration) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut coin_ids = Vec::new();
        for coin_id in &ids {
            let cache_key = CacheKey::CoinInfoUpdate(coin_id);
            if self.processor.cacher.can_process_now(&cache_key.key(), interval.as_secs()).await? {
                coin_ids.push(coin_id.clone());
            }
        }
        self.stream_producer.publish_update_coin_info(coin_ids).await
    }
}
