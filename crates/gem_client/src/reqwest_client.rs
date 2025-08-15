use crate::{Client, ClientError};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub struct ReqwestClient {
    base_url: String,
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new(url: String, client: reqwest::Client) -> Self {
        Self { base_url: url, client }
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    async fn send_request<R>(&self, response: reqwest::Response) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let status = response.status();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| ClientError::Network(format!("Failed to read response body: {}", e)))?;

        if status.is_success() {
            serde_json::from_slice(&body_bytes)
                .map_err(|e| ClientError::Serialization(format!("Failed to deserialize response: {}", e)))
        } else {
            let body_text = String::from_utf8_lossy(&body_bytes).to_string();
            Err(ClientError::Http {
                status: status.as_u16(),
                body: body_text,
            })
        }
    }

    fn map_reqwest_error(e: reqwest::Error) -> ClientError {
        if e.is_timeout() {
            ClientError::Timeout
        } else if e.is_connect() {
            ClientError::Network(format!("Connection error: {}", e))
        } else {
            ClientError::Network(e.to_string())
        }
    }
}

#[async_trait]
impl Client for ReqwestClient {
    async fn get<R>(&self, path: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(Self::map_reqwest_error)?;

        self.send_request(response).await
    }

    async fn post<T, R>(&self, path: &str, body: &T) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let json_body = serde_json::to_vec(body)
            .map_err(|e| ClientError::Serialization(format!("Failed to serialize request: {}", e)))?;

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await
            .map_err(Self::map_reqwest_error)?;

        self.send_request(response).await
    }
}
