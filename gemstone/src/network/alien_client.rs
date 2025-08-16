use crate::network::{AlienError, AlienProvider, AlienTarget};
use async_trait::async_trait;
use gem_client::{Client, ClientError, ContentType};
use primitives::Chain;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, str::FromStr, sync::Arc};

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

    async fn post<T, R>(&self, path: &str, body: &T, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let url = self.build_url(path);

        let mut request_headers = HashMap::from([("Content-Type".to_string(), ContentType::ApplicationJson.as_str().to_string())]);

        if let Some(provided_headers) = headers {
            request_headers.extend(provided_headers);
        }

        let content_type = request_headers.get("Content-Type").and_then(|s| ContentType::from_str(s).ok());

        let data = match content_type {
            Some(ContentType::TextPlain) => {
                let json_value = serde_json::to_value(body)?;
                match json_value {
                    serde_json::Value::String(s) => s.into_bytes(),
                    _ => return Err(ClientError::Serialization("Expected string body for text/plain content-type".to_string())),
                }
            }
            _ => serde_json::to_vec(body)?,
        };

        let target = AlienTarget {
            url,
            method: crate::network::AlienHttpMethod::Post,
            headers: Some(request_headers),
            body: Some(data),
        };

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
