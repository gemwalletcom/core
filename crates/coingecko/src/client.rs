use crate::model::{
    Coin, CoinIds, CoinInfo, CoinMarket, CoinQuery, CointListQuery, Data, ExchangeRates, Global, MarketChart, SearchTrending, TopGainersLosers,
};
use primitives::FiatRate;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use std::error::Error;

const COINGECKO_API_URL: &str = "https://api.coingecko.com";
const COINGECKO_API_PRO_URL: &str = "https://pro-api.coingecko.com";
static USER_AGENT_VALUE: HeaderValue =
    HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36");

#[derive(Clone)]
pub struct CoinGeckoClient {
    client: ClientWithMiddleware,
    url: String,
    api_key: String,
}

impl CoinGeckoClient {
    pub fn new(api_key: &str) -> Self {
        let client = ClientBuilder::new(reqwest::Client::new()).build();
        let url = Self::url_for_api_key(api_key.to_string().clone());
        Self {
            client,
            url,
            api_key: api_key.to_string(),
        }
    }

    pub fn new_with_client_middleware(client: ClientWithMiddleware, api_key: &str) -> Self {
        let url = Self::url_for_api_key(api_key.to_string().clone());
        Self {
            client,
            url,
            api_key: api_key.to_string(),
        }
    }

    fn url_for_api_key(api_key: String) -> String {
        if !api_key.is_empty() {
            return COINGECKO_API_PRO_URL.to_string();
        }
        COINGECKO_API_URL.to_string()
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, USER_AGENT_VALUE.clone());
        if !self.api_key.is_empty() {
            headers.insert("x-cg-pro-api-key", HeaderValue::from_str(self.api_key.as_str()).unwrap());
        }
        headers
    }

    pub async fn get_global(&self) -> Result<Global, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v3/global", self.url);
        Ok(self.client.get(&url).headers(self.headers()).send().await?.json::<Data<Global>>().await?.data)
    }

    pub async fn get_search_trending(&self) -> Result<SearchTrending, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v3/search/trending", self.url);
        Ok(self.client.get(&url).headers(self.headers()).send().await?.json().await?)
    }

    pub async fn get_top_gainers_losers(&self) -> Result<TopGainersLosers, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v3/coins/top_gainers_losers?vs_currency=usd", self.url);
        Ok(self.client.get(&url).headers(self.headers()).send().await?.json().await?)
    }

    pub async fn get_coin_list(&self) -> Result<Vec<Coin>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v3/coins/list", self.url);
        let query = CointListQuery { include_platform: true };
        Ok(self.client.get(&url).query(&query).headers(self.headers()).send().await?.json().await?)
    }

    pub async fn get_coin_list_new(&self) -> Result<CoinIds, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v3/coins/list/new", self.url);
        Ok(self.client.get(&url).headers(self.headers()).send().await?.json().await?)
    }

    pub async fn get_coin_markets(&self, page: usize, per_page: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page={}&page={}&sparkline=false&locale=en",
            self.url, per_page, page
        );
        Ok(self.client.get(&url).headers(self.headers()).send().await?.json().await?)
    }

    pub async fn get_coin_markets_ids(&self, ids: Vec<String>, per_page: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}/api/v3/coins/markets?vs_currency=usd&ids={}&order=market_cap_desc&sparkline=false&locale=en&per_page={}",
            self.url,
            ids.join(","),
            per_page
        );
        Ok(self.client.get(&url).headers(self.headers()).send().await?.json().await?)
    }

    pub async fn get_coin_markets_id(&self, id: &str) -> Result<CoinMarket, Box<dyn Error + Send + Sync>> {
        let markets = self.get_coin_markets_ids(vec![id.to_string()], 1).await?;
        markets.first().cloned().ok_or_else(|| format!("market {} not found", id).into())
    }

    pub async fn get_coin(&self, id: &str) -> Result<CoinInfo, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v3/coins/{}", self.url, id);
        let query = CoinQuery {
            market_data: false,
            community_data: true,
            tickers: false,
            localization: true,
            developer_data: true,
        };
        Ok(self.client.get(&url).query(&query).headers(self.headers()).send().await?.json().await?)
    }

    pub async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v3/exchange_rates", self.url);
        let rates = self.client.get(&url).headers(self.headers()).send().await?.json::<ExchangeRates>().await?;
        let fiat_rates: Vec<FiatRate> = rates
            .rates
            .into_iter()
            .filter(|x| x.1.rate_type == "fiat")
            .map(|x| FiatRate {
                symbol: x.0.to_uppercase(),
                name: x.1.name,
                rate: x.1.value,
            })
            .collect();
        Ok(fiat_rates)
    }

    pub async fn get_all_coin_markets(&self, per_page: usize, pages: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let mut all_coin_markets = Vec::new();
        let mut page = 1;

        loop {
            let coin_markets = self.get_coin_markets(page, per_page).await?;

            all_coin_markets.extend(coin_markets.clone());

            if coin_markets.is_empty() || page == pages {
                break;
            }

            page += 1;
        }

        Ok(all_coin_markets)
    }

    pub async fn get_market_chart(&self, coin_id: &str) -> Result<MarketChart, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}/api/v3/coins/{}/market_chart?vs_currency=usd&days=max&interval=daily&precision=full",
            self.url, coin_id
        );
        Ok(self.client.get(url).send().await?.json().await?)
    }
}
