use super::model::{PaybisAssetPair, PaybisQuoteRequest, PaybisQuoteResponse};
use reqwest::Client; // Use super to access model.rs in the same dir

#[derive(Debug, Clone)]
pub struct PaybisClient {
    client: Client,
    url: String,
    api_key: String,
}

impl PaybisClient {
    pub fn new(client: Client, url: String, api_key: String) -> Self {
        Self { client, url, api_key }
    }

    pub async fn get_quote(&self, request: PaybisQuoteRequest) -> Result<PaybisQuoteResponse, reqwest::Error> {
        let url = format!("{}/v1/quote", self.url);
        let response = self.client.post(&url).header("X-API-Key", &self.api_key).json(&request).send().await?;

        // Check for API errors returned in the JSON body even with a 2xx status
        // This part needs to be adapted if Paybis returns errors in a specific JSON structure for 2xx responses
        // For now, relying on error_for_status() for HTTP level errors (4xx, 5xx)
        response.error_for_status()?.json().await
    }

    pub async fn get_buy_currency_pairs(&self) -> Result<Vec<PaybisAssetPair>, reqwest::Error> {
        let url = format!("{}/v2/currency/pairs/buy-crypto", self.url);
        let response = self.client.get(&url).header("X-API-Key", &self.api_key).send().await?;
        response.error_for_status()?.json().await
    }

    pub async fn get_sell_currency_pairs(&self) -> Result<Vec<PaybisAssetPair>, reqwest::Error> {
        let url = format!("{}/v2/currency/pairs/sell-crypto", self.url);
        let response = self.client.get(&url).header("X-API-Key", &self.api_key).send().await?;
        response.error_for_status()?.json().await
    }
}
