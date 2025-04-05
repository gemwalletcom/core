use crate::model::{AltSeasonData, ApiResponse, FearGreedData};
use chrono::{Datelike, TimeZone};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use std::error::Error;

const COINMARKETCAP_API_URL: &str = "https://api.coinmarketcap.com";

#[derive(Clone)]
pub struct CoinMarketCapClient {
    client: ClientWithMiddleware,
    url: String,
    api_key: String,
}

impl CoinMarketCapClient {
    pub fn new(api_key: &str) -> Self {
        let client = ClientBuilder::new(reqwest::Client::new()).build();
        Self {
            client,
            url: COINMARKETCAP_API_URL.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn new_with_client_middleware(client: ClientWithMiddleware, api_key: &str) -> Self {
        Self {
            client,
            url: COINMARKETCAP_API_URL.to_string(),
            api_key: api_key.to_string(),
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if !self.api_key.is_empty() {
            headers.insert("X-CMC_PRO_API_KEY", HeaderValue::from_str(self.api_key.as_str()).unwrap());
        }
        headers
    }

    /// Get fear and greed chart data for a specific time range
    ///
    /// # Arguments
    /// * `start` - Start timestamp in seconds
    /// * `end` - End timestamp in seconds
    pub async fn get_fear_greed_data(&self, start: i64, end: i64) -> Result<FearGreedData, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/data-api/v3/fear-greed/chart", self.url);

        let response = self
            .client
            .get(&url)
            .query(&[("start", start.to_string()), ("end", end.to_string())])
            .headers(self.headers())
            .send()
            .await?
            .json::<ApiResponse<FearGreedData>>()
            .await?;

        Ok(response.data)
    }

    /// Get the latest fear and greed index data
    pub async fn get_latest_fear_greed(&self) -> Result<FearGreedData, Box<dyn Error + Send + Sync>> {
        let now = chrono::Utc::now();
        // End of current day
        let end = chrono::Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 23, 59, 59)
            .unwrap()
            .timestamp();

        // Start from 7 days ago, beginning of day
        let seven_days_ago = now - chrono::Duration::days(7);
        let start = chrono::Utc
            .with_ymd_and_hms(seven_days_ago.year(), seven_days_ago.month(), seven_days_ago.day(), 0, 0, 0)
            .unwrap()
            .timestamp();

        self.get_fear_greed_data(start, end).await
    }

    /// Get alt season index chart data for a specific time range
    ///
    /// # Arguments
    /// * `start` - Start timestamp in seconds
    /// * `end` - End timestamp in seconds
    pub async fn get_alt_season_data(&self, start: i64, end: i64) -> Result<AltSeasonData, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/data-api/v3/altcoin-season/chart", self.url);

        let response = self
            .client
            .get(&url)
            .query(&[("start", start.to_string()), ("end", end.to_string())])
            .headers(self.headers())
            .send()
            .await?
            .json::<ApiResponse<AltSeasonData>>()
            .await?;

        Ok(response.data)
    }

    /// Get the latest alt season index data
    pub async fn get_latest_alt_season(&self) -> Result<AltSeasonData, Box<dyn Error + Send + Sync>> {
        let now = chrono::Utc::now();
        // End of current day
        let end = chrono::Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 23, 59, 59)
            .unwrap()
            .timestamp();

        // Start from 7 days ago, beginning of day
        let seven_days_ago = now - chrono::Duration::days(7);
        let start = chrono::Utc
            .with_ymd_and_hms(seven_days_ago.year(), seven_days_ago.month(), seven_days_ago.day(), 0, 0, 0)
            .unwrap()
            .timestamp();

        self.get_alt_season_data(start, end).await
    }
}
