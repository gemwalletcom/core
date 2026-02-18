mod content_type;
mod types;

#[cfg(feature = "testkit")]
pub mod testkit;

#[cfg(feature = "reqwest")]
mod reqwest_client;

#[cfg(feature = "reqwest")]
pub mod retry;

#[cfg(feature = "reqwest")]
pub mod client_config;

pub mod query;

pub use content_type::{CONTENT_TYPE, ContentType};
pub use query::build_path_with_query;
pub use types::{ClientError, Response, decode_json_byte_array, deserialize_response};

#[cfg(feature = "reqwest")]
pub use reqwest_client::ReqwestClient;

#[cfg(feature = "reqwest")]
pub use retry::{default_should_retry, retry, retry_policy};

#[cfg(feature = "reqwest")]
pub use client_config::builder;

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashMap, fmt::Debug};

pub type Data = Vec<u8>;
pub const X_CACHE_TTL: &str = "x-cache-ttl";

#[async_trait]
pub trait Client: Send + Sync + Debug {
    async fn get<R>(&self, path: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned;
    async fn get_with_headers<R>(&self, path: &str, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let _ = headers;
        self.get(path).await
    }
    async fn post<T, R>(&self, path: &str, body: &T, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned;
}
