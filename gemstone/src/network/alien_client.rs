use crate::network::{AlienError, AlienProvider, AlienTarget};
use async_trait::async_trait;
use gem_client::{Client, ClientError};
use primitives::Chain;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

#[derive(Debug)]
pub struct AlienClient {
    base_url: String,
    provider: Arc<dyn AlienProvider>,
}

impl AlienClient {
    pub fn new(base_url: String, provider: Arc<dyn AlienProvider>) -> Self {
        Self { base_url, provider }
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }
}

#[async_trait]
impl Client for AlienClient {
    async fn get<R>(&self, path: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let target = AlienTarget::get(&url);

        let response_data = self
            .provider
            .request(target)
            .await
            .map_err(|e| ClientError::Network(format!("Alien provider error: {e}")))?;

        serde_json::from_slice(&response_data).map_err(|e| ClientError::Serialization(format!("Failed to deserialize response: {e}")))
    }

    async fn post<T, R>(&self, path: &str, body: &T) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let json_value = serde_json::to_value(body).map_err(|e| ClientError::Serialization(format!("Failed to serialize request: {e}")))?;

        let target = AlienTarget::post_json(&url, json_value);

        let response_data = self
            .provider
            .request(target)
            .await
            .map_err(|e| ClientError::Network(format!("Alien provider error: {e}")))?;

        serde_json::from_slice(&response_data).map_err(|e| ClientError::Serialization(format!("Failed to deserialize response: {e}")))
    }
}

#[async_trait]
impl AlienProvider for AlienClient {
    async fn request(&self, target: AlienTarget) -> Result<Vec<u8>, AlienError> {
        self.provider.request(target).await
    }

    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Vec<u8>>, AlienError> {
        self.provider.batch_request(targets).await
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        self.provider.get_endpoint(chain)
    }
}
