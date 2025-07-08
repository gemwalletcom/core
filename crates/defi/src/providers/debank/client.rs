use reqwest::Client;

use super::models::DeBankProtocol;
use crate::{error::DeFiError, providers::debank::DeBankChain};

pub struct DeBankClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl DeBankClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://pro-openapi.debank.com".to_string(),
        }
    }

    async fn request<T: serde::de::DeserializeOwned>(&self, endpoint: &str, params: Vec<(&str, &str)>) -> Result<T, DeFiError> {
        let mut headers = reqwest::header::HeaderMap::new();

        headers.insert(
            "AccessKey",
            self.api_key.parse().map_err(|_| DeFiError::AuthError("Invalid API key format".to_string()))?,
        );

        let url = format!("{}{}", self.base_url, endpoint);
        let response = self.client.get(&url).headers(headers).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(DeFiError::NetworkError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }
        let result = response.json::<T>().await?;
        Ok(result)
    }

    pub async fn get_complex_protocol_list(&self, address: &str, chain_ids: &str) -> Result<Vec<DeBankProtocol>, DeFiError> {
        let params = vec![("id", address), ("chain_ids", chain_ids)];
        self.request("/v1/user/all_complex_protocol_list", params).await
    }

    pub async fn get_used_chain_list(&self, address: &str) -> Result<Vec<DeBankChain>, DeFiError> {
        let params = vec![("id", address)];
        self.request("/v1/user/used_chain_list", params).await
    }
}
