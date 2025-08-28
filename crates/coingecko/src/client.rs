use crate::model::{
    Coin, CoinGeckoResponse, CoinIds, CoinInfo, CoinMarket, CoinQuery, CointListQuery, Data, ExchangeRates, Global, MarketChart, SearchTrending,
    TopGainersLosers,
};
use gem_client::{build_path_with_query, retry, Client, ReqwestClient};
use primitives::{FiatRate, DEFAULT_FIAT_CURRENCY};
use reqwest::header::{HeaderMap, HeaderValue};
use std::error::Error;

pub const COINGECKO_API_HOST: &str = "api.coingecko.com";
pub const COINGECKO_API_PRO_HOST: &str = "pro-api.coingecko.com";
static COINGECKO_API_HEADER_KEY: &str = "x-cg-pro-api-key";
static USER_AGENT_VALUE: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

#[derive(Debug, Clone)]
pub struct CoinGeckoClient<C: Client> {
    client: C,
}

impl CoinGeckoClient<ReqwestClient> {
    pub fn new(api_key: &str) -> Self {
        let url = Self::url_for_api_key(api_key.to_string());
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        if !api_key.is_empty() {
            headers.insert(COINGECKO_API_HEADER_KEY, HeaderValue::from_str(api_key).unwrap());
        }
        let client_builder = reqwest::Client::builder().default_headers(headers);

        let reqwest_client = client_builder.build().unwrap();
        let client = ReqwestClient::new(url, reqwest_client);
        Self { client }
    }

    fn url_for_api_key(api_key: String) -> String {
        let host = if !api_key.is_empty() { COINGECKO_API_PRO_HOST } else { COINGECKO_API_HOST };
        format!("https://{}", host)
    }
}

impl<C: Client> CoinGeckoClient<C> {
    pub fn new_with_client(client: C) -> Self {
        Self { client }
    }

    async fn _get<T>(&self, path: &str) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        retry(
            || async {
                let response: CoinGeckoResponse<T> = self.client.get(path).await.map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;
                match response {
                    CoinGeckoResponse::Success(data) => Ok(data),
                    CoinGeckoResponse::Error(error) => Err(error.error.into()),
                }
            },
            3,
            None::<fn(&Box<dyn Error + Send + Sync>) -> bool>, // Use default retry behavior
        )
        .await
    }

    pub async fn get_global(&self) -> Result<Global, Box<dyn Error + Send + Sync>> {
        let path = "/api/v3/global";
        Ok(self.client.get::<Data<Global>>(path).await?.data)
    }

    pub async fn get_search_trending(&self) -> Result<SearchTrending, Box<dyn Error + Send + Sync>> {
        let path = "/api/v3/search/trending";
        self._get(path).await
    }

    pub async fn get_top_gainers_losers(&self) -> Result<TopGainersLosers, Box<dyn Error + Send + Sync>> {
        let path = "/api/v3/coins/top_gainers_losers?vs_currency=usd";
        Ok(self.client.get(path).await?)
    }

    pub async fn get_coin_list(&self) -> Result<Vec<Coin>, Box<dyn Error + Send + Sync>> {
        let query = CointListQuery { include_platform: true };
        let path = build_path_with_query("/api/v3/coins/list", &query)?;
        self._get(&path).await
    }

    pub async fn get_coin_list_new(&self) -> Result<CoinIds, Box<dyn Error + Send + Sync>> {
        let path = "/api/v3/coins/list/new";
        Ok(self.client.get(path).await?)
    }

    pub async fn get_coin_markets(&self, page: usize, per_page: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let path = format!(
            "/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page={}&page={}&sparkline=false&locale=en",
            per_page, page
        );
        Ok(self.client.get(&path).await?)
    }

    pub async fn get_coin_markets_ids(&self, ids: Vec<String>, per_page: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let path = format!(
            "/api/v3/coins/markets?vs_currency=usd&ids={}&order=market_cap_desc&sparkline=false&locale=en&per_page={}",
            ids.join(","),
            per_page
        );
        Ok(self.client.get(&path).await?)
    }

    pub async fn get_coin_markets_id(&self, id: &str) -> Result<CoinMarket, Box<dyn Error + Send + Sync>> {
        let markets = self.get_coin_markets_ids(vec![id.to_string()], 1).await?;
        markets.first().cloned().ok_or_else(|| format!("market {id} not found").into())
    }

    pub async fn get_coin(&self, id: &str) -> Result<CoinInfo, Box<dyn Error + Send + Sync>> {
        let query = CoinQuery {
            market_data: false,
            community_data: true,
            tickers: false,
            localization: true,
            developer_data: true,
        };
        let base_path = format!("/api/v3/coins/{}", id);
        let path = build_path_with_query(&base_path, &query)?;
        Ok(self.client.get(&path).await?)
    }

    pub async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        let path = "/api/v3/exchange_rates";
        let rates: ExchangeRates = self.client.get(path).await?;
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
        let path = format!(
            "/api/v3/coins/{}/market_chart?vs_currency=usd&days={}&interval={}&precision=full",
            coin_id, days, interval
        );
        Ok(self.client.get(&path).await?)
    }
}
