use crate::model::{
    Coin, CoinGeckoResponse, CoinIds, CoinInfo, CoinMarket, CoinMarketsQuery, CoinQuery, CointListQuery, Data, ExchangeRates, Global, MarketChart,
    MarketChartQuery, SearchTrending, TopGainersLosers,
};
use gem_client::{Client, ClientError, ReqwestClient};
use primitives::{FiatRate, DEFAULT_FIAT_CURRENCY};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::error::Error;

pub const COINGECKO_API_HOST: &str = "api.coingecko.com";
pub const COINGECKO_API_PRO_HOST: &str = "pro-api.coingecko.com";
pub const COINGECKO_API_KEY_HEADER: &str = "x-cg-pro-api-key";
static USER_AGENT_VALUE: HeaderValue =
    HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36");

#[derive(Clone)]
pub struct CoinGeckoClient {
    client: ReqwestClient,
}

impl CoinGeckoClient {
    pub fn new(api_key: &str) -> Self {
        let reqwest_client = Self::build_reqwest_client(api_key);
        let url = Self::url_for_api_key(api_key.to_string());
        let client = ReqwestClient::new(url, reqwest_client);
        Self { client }
    }

    pub fn new_with_reqwest_client(reqwest_client: reqwest::Client, api_key: &str) -> Self {
        let url = Self::url_for_api_key(api_key.to_string());
        let client = ReqwestClient::new(url, reqwest_client);
        Self { client }
    }

    fn build_reqwest_client(api_key: &str) -> reqwest::Client {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, USER_AGENT_VALUE.clone());

        if !api_key.is_empty() {
            headers.insert(COINGECKO_API_KEY_HEADER, HeaderValue::from_str(api_key).expect("Invalid API key"));
        }

        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client")
    }

    fn url_for_api_key(api_key: String) -> String {
        let host = if !api_key.is_empty() { COINGECKO_API_PRO_HOST } else { COINGECKO_API_HOST };
        format!("https://{}", host)
    }

    async fn handle_response<R>(&self, result: Result<CoinGeckoResponse<R>, ClientError>) -> Result<R, Box<dyn Error + Send + Sync>>
    where
        R: serde::de::DeserializeOwned,
    {
        match result {
            Ok(CoinGeckoResponse::Success(data)) => Ok(data),
            Ok(CoinGeckoResponse::Error(error)) => Err(format!("CoinGecko API error: {}", error.error).into()),
            Err(ClientError::Http { status, body }) => {
                // Try to parse as CoinGecko error response
                if let Ok(error_response) = serde_json::from_str::<crate::model::CoinGeckoErrorResponse>(&body) {
                    Err(format!("CoinGecko API error: {}", error_response.error).into())
                } else {
                    Err(format!("HTTP error {}: {}", status, body).into())
                }
            }
            Err(e) => Err(format!("Client error: {}", e).into()),
        }
    }

    pub async fn get_global(&self) -> Result<Global, Box<dyn Error + Send + Sync>> {
        let result = self.client.get("/api/v3/global", None::<&()>).await;
        let data: Data<Global> = self.handle_response(result).await?;
        Ok(data.data)
    }

    pub async fn get_search_trending(&self) -> Result<SearchTrending, Box<dyn Error + Send + Sync>> {
        let result = self.client.get("/api/v3/search/trending", None::<&()>).await;
        self.handle_response(result).await
    }

    pub async fn get_top_gainers_losers(&self) -> Result<TopGainersLosers, Box<dyn Error + Send + Sync>> {
        self.get_top_gainers_losers_with_currency("usd").await
    }

    pub async fn get_top_gainers_losers_with_currency(&self, vs_currency: &str) -> Result<TopGainersLosers, Box<dyn Error + Send + Sync>> {
        let query = [("vs_currency", vs_currency)];
        let result = self.client.get("/api/v3/coins/top_gainers_losers", Some(&query)).await;
        self.handle_response(result).await
    }

    pub async fn get_coin_list(&self) -> Result<Vec<Coin>, Box<dyn Error + Send + Sync>> {
        let query = CointListQuery { include_platform: true };
        let result = self.client.get("/api/v3/coins/list", Some(&query)).await;
        self.handle_response(result).await
    }

    pub async fn get_coin_list_new(&self) -> Result<CoinIds, Box<dyn Error + Send + Sync>> {
        let result = self.client.get("/api/v3/coins/list/new", None::<&()>).await;
        self.handle_response(result).await
    }

    pub async fn get_coin_markets(&self, page: usize, per_page: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let query = CoinMarketsQuery::new(page, per_page);
        let result = self.client.get("/api/v3/coins/markets", Some(&query)).await;
        self.handle_response(result).await
    }

    pub async fn get_coin_markets_ids(&self, ids: Vec<String>, per_page: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let query = CoinMarketsQuery::with_ids(ids, per_page);
        let result = self.client.get("/api/v3/coins/markets", Some(&query)).await;
        self.handle_response(result).await
    }

    pub async fn get_coin_markets_id(&self, id: &str) -> Result<CoinMarket, Box<dyn Error + Send + Sync>> {
        let markets = self.get_coin_markets_ids(vec![id.to_string()], 1).await?;
        markets.first().cloned().ok_or_else(|| format!("market {id} not found").into())
    }

    pub async fn get_coin(&self, id: &str) -> Result<CoinInfo, Box<dyn Error + Send + Sync>> {
        let path = format!("/api/v3/coins/{}", id);
        let query = CoinQuery {
            market_data: false,
            community_data: true,
            tickers: false,
            localization: true,
            developer_data: true,
        };
        let result = self.client.get(&path, Some(&query)).await;
        self.handle_response(result).await
    }

    pub async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        let result = self.client.get("/api/v3/exchange_rates", None::<&()>).await;
        let rates: ExchangeRates = self.handle_response(result).await?;
        let usd_rate = rates
            .rates
            .get(DEFAULT_FIAT_CURRENCY.to_lowercase().as_str())
            .ok_or("Default fiat currency rate not found")?
            .value;

        let fiat_rates: Vec<FiatRate> = rates
            .rates
            .clone()
            .into_iter()
            .filter(|x| x.1.rate_type == "fiat")
            .map(|x| FiatRate {
                symbol: x.0.to_uppercase(),
                rate: x.1.value / usd_rate,
            })
            .collect();
        Ok(fiat_rates)
    }

    pub async fn get_all_coin_markets(
        &self,
        start_page: Option<usize>,
        per_page: usize,
        pages: usize,
    ) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let mut all_coin_markets = Vec::new();
        let mut page = start_page.unwrap_or(1);

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

    pub async fn get_market_chart(&self, coin_id: &str, interval: &str, days: &str) -> Result<MarketChart, Box<dyn Error + Send + Sync>> {
        let path = format!("/api/v3/coins/{}/market_chart", coin_id);
        let query = MarketChartQuery::new(days, interval);
        let result = self.client.get(&path, Some(&query)).await;
        self.handle_response(result).await
    }
}
