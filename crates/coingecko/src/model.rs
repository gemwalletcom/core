use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Coin {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub platforms: HashMap<String, Option<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinInfo {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub asset_platform_id: Option<String>,
    pub preview_listing: bool,
    pub market_cap_rank: Option<i32>,
    pub watchlist_portfolio_users: Option<f32>,
    pub platforms: HashMap<String, String>,
    pub detail_platforms: HashMap<String, Option<DetailPlatform>>,
    pub links: CoinMarketLinks,
    pub community_data: Option<CommunityData>,
    pub image: Image,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    pub thumb: String,
    pub small: String,
    pub large: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityData {
    pub twitter_followers: Option<i64>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeveloperData {
    pub stars: Option<i64>,
    pub subscribers: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetailPlatform {
    pub decimal_place: Option<i32>,
    pub contract_address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketChart {
    pub prices: Vec<Vec<f64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinMarket {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub market_cap: Option<f64>,
    pub fully_diluted_valuation: Option<f64>,
    pub market_cap_rank: Option<i32>,
    pub total_volume: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub ath: Option<f64>,
    pub ath_date: Option<DateTime<Utc>>,
    pub atl: Option<f64>,
    pub atl_date: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CointListQuery {
    pub include_platform: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinQuery {
    pub market_data: bool,
    pub community_data: bool,
    pub tickers: bool,
    pub localization: bool,
    pub developer_data: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinMarketLinks {
    pub homepage: Vec<String>,
    pub blockchain_site: Vec<Option<String>>,
    pub chat_url: Vec<String>,
    pub subreddit_url: Option<String>,
    pub twitter_screen_name: Option<String>,
    pub facebook_username: Option<String>,
    pub telegram_channel_identifier: Option<String>,
    pub repos_url: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExchangeRates {
    pub rates: HashMap<String, ExchangeRate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExchangeRate {
    pub unit: String,
    #[serde(rename = "type")]
    pub rate_type: String,
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchTrending {
    pub coins: Vec<SearchTrendingItem>,
}

impl SearchTrending {
    pub fn get_coins_ids(&self) -> Vec<String> {
        self.coins.iter().map(|x| x.item.id.clone()).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchTrendingItem {
    pub item: SearchTrendingItemCoin,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchTrendingItemCoin {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchTrendingPrice {
    pub id: String,
}
