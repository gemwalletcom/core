use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use storage::models::FiatRate;
use std::collections::HashMap;

const COINGECKO_API_URL: &str = "https://api.coingecko.com";
const COINGECKO_API_PRO_URL: &str = "https://pro-api.coingecko.com";
static USER_AGENT_VALUE: HeaderValue = HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36");

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
    pub coingecko_rank: Option<i32>,
    pub community_score: f32,
    pub watchlist_portfolio_users: f32,
    pub liquidity_score: f32,
    //pub platforms: HashMap<String, Option<String>>,
    pub detail_platforms: HashMap<String, Option<DetailPlatform>>,
    pub links: CoinMarketLinks,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetailPlatform {
    pub decimal_place: Option<i32>,
    pub contract_address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinMarket {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_rank: Option<i32>,
    pub total_volume: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinMarketLinks {
    pub homepage: Vec<String>,
    pub blockchain_site: Vec<String>,
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

#[derive(Clone)]
pub struct CoinGeckoClient {
    client: reqwest::Client,
    url: String,
    api_key: String,
}

impl CoinGeckoClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        let url = Self::url_for_api_key(api_key.clone());
        Self { client, url, api_key }
    }

    fn url_for_api_key(api_key: String) -> String {
        if !api_key.is_empty() {
            return COINGECKO_API_PRO_URL.to_string()
        }
        COINGECKO_API_URL.to_string()
    }

    pub fn convert_coin_vec_to_map(coins: Vec<Coin>) -> HashMap<String, Coin> {
        coins.into_iter().map(|coin| (coin.id.clone(), coin)).collect()
    }
    
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, USER_AGENT_VALUE.clone());
        headers
    }

    pub async fn get_coin_list(&self) -> Result<Vec<Coin>, Error> {
        let url = format!("{}/api/v3/coins/list?include_platform=true&x_cg_pro_api_key={}", self.url, self.api_key);
        let response = self.client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?;
        let coins: Vec<Coin> = response.json().await?;
        Ok(coins)
    }

    pub async fn get_coin_markets(&self, page: u32, per_page: u32) -> Result<Vec<CoinMarket>, Error> {
        let url = format!(
            "{}/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page={}&page={}&sparkline=false&locale=en&x_cg_pro_api_key={}",
            self.url, per_page, page, self.api_key
        );
        let response = self.client
            .get(&url)
            .headers(self.headers())
            .send().await?;

        let coin_markets: Vec<CoinMarket> = response.json().await?;
        Ok(coin_markets)
    }

    pub async fn get_coin(&self, coin: &str) -> Result<CoinInfo, Error> {
        let url = format!("{}/api/v3/coins/{}?x_cg_pro_api_key={}&market_data=false&community_data=false&tickers=false&localization=false&developer_data=false", self.url, coin, self.api_key);

        //println!("url: {}", url);

        let response = self.client
            .get(&url)
            .headers(self.headers())
            .send().await?;

        let coin: CoinInfo = response.json().await?;
        Ok(coin)
    }

    pub async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Error> {
        let url = format!("{}/api/v3/exchange_rates?x_cg_pro_api_key={}",self.url, self.api_key);
        let response = self.client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?;

        let rates: ExchangeRates = response.json().await?;

        let fiat_rates: Vec<FiatRate> = rates.rates
            .into_iter()
            .filter(|x| x.1.rate_type == "fiat")
            .map(|x| {
                FiatRate{symbol: x.0.to_uppercase(), name: x.1.name, rate: x.1.value}
            })
            .collect();
        Ok(fiat_rates)
    }

    pub async fn get_all_coin_markets(&self, per_page: u32, limit_page: u32) -> Result<Vec<CoinMarket>, Error> {
        let mut all_coin_markets = Vec::new();
        let mut page = 1;

        loop {
            let coin_markets = self.get_coin_markets(page, per_page).await?;
            if coin_markets.is_empty() || page == limit_page  {
                break;
            }
            all_coin_markets.extend(coin_markets);
            page += 1;
        }

        Ok(all_coin_markets)
    }
}
