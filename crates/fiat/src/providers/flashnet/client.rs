use std::error::Error;

use primitives::FiatProviderName;
use reqwest::Client;

use super::model::{FlashnetEstimateResponse, FlashnetOnrampRequest, FlashnetOnrampResponse, FlashnetRoutesResponse, FlashnetStatusResponse};

pub struct FlashnetClient {
    client: Client,
    base_url: String,
    api_key: String,
    pub(crate) affiliate_id: String,
}

impl FlashnetClient {
    pub const NAME: FiatProviderName = FiatProviderName::Flashnet;

    pub fn new(client: Client, base_url: String, api_key: String, affiliate_id: String) -> Self {
        Self {
            client,
            base_url,
            api_key,
            affiliate_id,
        }
    }

    pub async fn get_routes(&self) -> Result<FlashnetRoutesResponse, Box<dyn Error + Send + Sync>> {
        let response = self.client.get(format!("{}/v1/orchestration/routes", self.base_url)).send().await?.error_for_status()?;
        Ok(response.json::<FlashnetRoutesResponse>().await?)
    }

    pub async fn create_onramp(&self, request: FlashnetOnrampRequest, idempotency_key: &str) -> Result<FlashnetOnrampResponse, Box<dyn Error + Send + Sync>> {
        let response = self
            .client
            .post(format!("{}/v1/orchestration/onramp", self.base_url))
            .bearer_auth(&self.api_key)
            .header("X-Idempotency-Key", idempotency_key)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<FlashnetOnrampResponse>().await?)
    }

    pub async fn get_estimate(&self, destination_chain: &str, destination_asset: &str, amount: &str) -> Result<FlashnetEstimateResponse, Box<dyn Error + Send + Sync>> {
        let response = self
            .client
            .get(format!("{}/v1/orchestration/estimate", self.base_url))
            .bearer_auth(&self.api_key)
            .query(&[
                ("sourceChain", "spark"),
                ("sourceAsset", "USDB"),
                ("destinationChain", destination_chain),
                ("destinationAsset", destination_asset),
                ("amount", amount),
                ("affiliateId", self.affiliate_id.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json::<FlashnetEstimateResponse>().await?)
    }

    pub async fn get_order_status(&self, order_id: &str) -> Result<FlashnetStatusResponse, Box<dyn Error + Send + Sync>> {
        let response = self
            .client
            .get(format!("{}/v1/orchestration/status", self.base_url))
            .bearer_auth(&self.api_key)
            .query(&[("id", order_id)])
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<FlashnetStatusResponse>().await?)
    }
}
