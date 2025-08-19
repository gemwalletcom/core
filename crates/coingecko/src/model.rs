use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Global {
    pub market_cap_change_percentage_24h_usd: f64,
    pub total_market_cap: Total,
    pub total_volume: Total,
    pub market_cap_percentage: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Total {
    pub usd: f64,
}

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
pub struct CoinMarketsQuery {
    pub vs_currency: String,
    pub order: String,
    pub per_page: usize,
    pub page: usize,
    pub sparkline: bool,
    pub locale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<String>,
}

impl CoinMarketsQuery {
    pub fn new(page: usize, per_page: usize) -> Self {
        Self {
            vs_currency: "usd".to_string(),
            order: "market_cap_desc".to_string(),
            per_page,
            page,
            sparkline: false,
            locale: "en".to_string(),
            ids: None,
        }
    }

    pub fn with_ids(ids: Vec<String>, per_page: usize) -> Self {
        Self {
            vs_currency: "usd".to_string(),
            order: "market_cap_desc".to_string(),
            per_page,
            page: 1,
            sparkline: false,
            locale: "en".to_string(),
            ids: Some(ids.join(",")),
        }
    }

    pub fn with_currency(mut self, currency: &str) -> Self {
        self.vs_currency = currency.to_string();
        self
    }

    pub fn with_order(mut self, order: &str) -> Self {
        self.order = order.to_string();
        self
    }

    pub fn with_sparkline(mut self, sparkline: bool) -> Self {
        self.sparkline = sparkline;
        self
    }

    pub fn with_locale(mut self, locale: &str) -> Self {
        self.locale = locale.to_string();
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketChartQuery {
    pub vs_currency: String,
    pub days: String,
    pub interval: String,
    pub precision: String,
}

impl MarketChartQuery {
    pub fn new(days: &str, interval: &str) -> Self {
        Self {
            vs_currency: "usd".to_string(),
            days: days.to_string(),
            interval: interval.to_string(),
            precision: "full".to_string(),
        }
    }

    pub fn with_currency(mut self, currency: &str) -> Self {
        self.vs_currency = currency.to_string();
        self
    }

    pub fn with_precision(mut self, precision: &str) -> Self {
        self.precision = precision.to_string();
        self
    }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopGainersLosers {
    pub top_gainers: CoinIds,
    pub top_losers: CoinIds,
}

impl TopGainersLosers {
    pub fn get_gainers_ids(&self) -> Vec<String> {
        self.top_gainers.ids()
    }

    pub fn get_losers_ids(&self) -> Vec<String> {
        self.top_losers.ids()
    }
}

impl SearchTrending {
    pub fn get_coins_ids(&self) -> Vec<String> {
        self.coins.iter().map(|x| x.item.id.clone()).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchTrendingItem {
    pub item: CoinId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinId {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinIds(pub Vec<CoinId>);

impl CoinIds {
    pub fn ids(&self) -> Vec<String> {
        self.0.iter().map(|x| x.id.clone()).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinGeckoErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CoinGeckoResponse<T> {
    Success(T),
    Error(CoinGeckoErrorResponse),
}
