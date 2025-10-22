use async_trait::async_trait;
use gem_client::{Client, ClientError, ContentType};
use primitives::Chain;
use serde::{Serialize, de::DeserializeOwned};
use serde_json;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
    sync::Arc,
};

pub const X_CACHE_TTL: &str = "x-cache-ttl";

#[derive(Debug, Clone)]
pub struct RpcResponse {
    pub status: Option<u16>,
    pub data: Vec<u8>,
}

pub trait RpcClientError: Error + Send + Sync + 'static + Display + Sized {
    fn into_client_error(self) -> ClientError {
        ClientError::Network(format!("RPC provider error: {}", self))
    }
}

#[derive(Debug, Clone)]
pub struct Target {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

impl Target {
    pub fn get(url: &str) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            headers: None,
            body: None,
        }
    }

    pub fn post_json(url: &str, body: serde_json::Value) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Post,
            headers: Some(HashMap::from([("Content-Type".into(), "application/json".into())])),
            body: Some(serde_json::to_vec(&body).expect("Failed to serialize JSON body")),
        }
    }

    pub fn set_cache_ttl(mut self, ttl: u64) -> Self {
        if self.headers.is_none() {
            self.headers = Some(HashMap::new());
        }
        if let Some(headers) = self.headers.as_mut() {
            headers.insert(X_CACHE_TTL.into(), ttl.to_string());
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

impl From<HttpMethod> for String {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Patch => "PATCH",
        }
        .into()
    }
}

#[async_trait]
pub trait RpcProvider: Send + Sync + Debug {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error>;
    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct RpcClient<E> {
    base_url: String,
    provider: Arc<dyn RpcProvider<Error = E>>,
}

impl<E> RpcClient<E>
where
    E: RpcClientError,
{
    pub fn new(base_url: String, provider: Arc<dyn RpcProvider<Error = E>>) -> Self {
        Self { base_url, provider }
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }
}

#[async_trait]
impl<E> Client for RpcClient<E>
where
    E: RpcClientError,
{
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
        let target = if let Some(headers) = headers {
            Target {
                url,
                method: HttpMethod::Get,
                headers: Some(headers),
                body: None,
            }
        } else {
            Target::get(&url)
        };

        let response = self.provider.request(target).await.map_err(|e| e.into_client_error())?;

        serde_json::from_slice(&response.data).map_err(|e| ClientError::Serialization(format!("Failed to deserialize response: {e}")))
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
            Some(ContentType::TextPlain) | Some(ContentType::ApplicationFormUrlEncoded) => {
                let json_value = serde_json::to_value(body)?;
                match json_value {
                    serde_json::Value::String(s) => s.into_bytes(),
                    _ => return Err(ClientError::Serialization("Expected string body for text/plain content-type".to_string())),
                }
            }
            Some(ContentType::ApplicationXBinary) => {
                let json_value = serde_json::to_value(body)?;
                match json_value {
                    serde_json::Value::String(s) => hex::decode(&s).map_err(|e| ClientError::Serialization(format!("Failed to decode hex string: {e}")))?,
                    _ => return Err(ClientError::Serialization("Expected hex string body for binary content-type".to_string())),
                }
            }
            _ => serde_json::to_vec(body)?,
        };

        let target = Target {
            url,
            method: HttpMethod::Post,
            headers: Some(request_headers),
            body: Some(data),
        };

        let response = self.provider.request(target).await.map_err(|e| e.into_client_error())?;

        serde_json::from_slice(&response.data).map_err(|e| ClientError::Serialization(format!("Failed to deserialize response: {e}")))
    }
}

#[async_trait]
impl<E> RpcProvider for RpcClient<E>
where
    E: RpcClientError,
{
    type Error = E;

    async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error> {
        self.provider.request(target).await
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error> {
        self.provider.get_endpoint(chain)
    }
}
