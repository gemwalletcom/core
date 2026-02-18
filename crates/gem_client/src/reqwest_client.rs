use crate::{CONTENT_TYPE, Client, ClientError, ContentType, Response, deserialize_response, retry_policy};
use async_trait::async_trait;
use reqwest::RequestBuilder;
use reqwest::header::USER_AGENT;
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashMap, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub struct ReqwestClient {
    base_url: String,
    client: reqwest::Client,
    user_agent: Option<String>,
}

impl ReqwestClient {
    pub fn new(url: String, client: reqwest::Client) -> Self {
        Self {
            base_url: url,
            client,
            user_agent: None,
        }
    }

    pub fn new_with_user_agent(url: String, client: reqwest::Client, user_agent: String) -> Self {
        Self {
            base_url: url,
            client,
            user_agent: Some(user_agent),
        }
    }

    pub fn new_with_retry(url: String, timeout_secs: u64, max_retries: u32) -> Self {
        let client = crate::client_config::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .retry(retry_policy(url.clone(), max_retries))
            .build()
            .expect("Failed to build reqwest client with retry");
        Self {
            base_url: url,
            client,
            user_agent: None,
        }
    }

    pub fn new_test_client(url: String) -> Self {
        Self::new_with_retry(url, 30, 3)
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    fn build_request(&self, request: RequestBuilder, headers: Option<HashMap<String, String>>) -> RequestBuilder {
        let request = if let Some(ref user_agent) = self.user_agent {
            request.header(USER_AGENT, user_agent)
        } else {
            request
        };

        if let Some(headers) = headers {
            headers.into_iter().fold(request, |req, (key, value)| req.header(&key, &value))
        } else {
            request
        }
    }

    async fn send_request<R>(&self, response: reqwest::Response) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let status = response.status().as_u16();
        let data = response
            .bytes()
            .await
            .map_err(|e| ClientError::Network(format!("Failed to read response body: {e}")))?
            .to_vec();

        let response = Response { status: Some(status), data };
        deserialize_response(&response)
    }

    fn map_reqwest_error(e: reqwest::Error) -> ClientError {
        if e.is_timeout() {
            ClientError::Timeout
        } else if e.is_connect() {
            ClientError::Network(format!("Connection error: {e}"))
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
        self.get_with_headers(path, None).await
    }

    async fn get_with_headers<R>(&self, path: &str, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let request = self.build_request(self.client.get(&url), headers);

        let response = request.send().await.map_err(Self::map_reqwest_error)?;
        self.send_request(response).await
    }

    async fn post<T, R>(&self, path: &str, body: &T, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let headers = headers.unwrap_or_else(|| HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationJson.as_str().to_string())]));

        let content_type = headers.get(CONTENT_TYPE).and_then(|s| ContentType::from_str(s).ok());

        let request_body = match content_type {
            Some(ContentType::TextPlain) | Some(ContentType::ApplicationFormUrlEncoded) | Some(ContentType::ApplicationXBinary) | Some(ContentType::ApplicationAptosBcs) => {
                let json_value = serde_json::to_value(body).map_err(|e| ClientError::Serialization(format!("Failed to serialize request: {e}")))?;
                match json_value {
                    serde_json::Value::String(s) => {
                        if matches!(content_type, Some(ContentType::ApplicationXBinary) | Some(ContentType::ApplicationAptosBcs)) {
                            // For binary content, decode hex string to bytes
                            hex::decode(&s).map_err(|e| ClientError::Serialization(format!("Failed to decode hex string: {e}")))?
                        } else {
                            s.into_bytes()
                        }
                    }
                    serde_json::Value::Array(values) if matches!(content_type, Some(ContentType::ApplicationXBinary) | Some(ContentType::ApplicationAptosBcs)) => {
                        crate::decode_json_byte_array(values)?
                    }
                    _ => return Err(ClientError::Serialization("Expected string body for text/plain or binary content-type".to_string())),
                }
            }
            _ => serde_json::to_vec(body).map_err(|e| ClientError::Serialization(format!("Failed to serialize request: {e}")))?,
        };

        let request = self.build_request(self.client.post(&url).body(request_body), Some(headers));

        let response = request.send().await.map_err(Self::map_reqwest_error)?;

        self.send_request(response).await
    }
}
