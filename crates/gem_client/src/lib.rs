mod content_type;
mod types;

#[cfg(feature = "reqwest")]
mod reqwest_client;

pub mod rpc;

#[cfg(feature = "reqwest")]
pub mod retry;

#[cfg(feature = "reqwest")]
pub mod client_config;

pub mod query;

pub use content_type::{ContentType, CONTENT_TYPE};
pub use query::build_path_with_query;
pub use types::ClientError;
pub use rpc::{HttpMethod, RpcClient, RpcProvider, Target, X_CACHE_TTL};

#[cfg(feature = "reqwest")]
pub use reqwest_client::ReqwestClient;

#[cfg(feature = "reqwest")]
pub use retry::{default_should_retry, retry, retry_policy};

#[cfg(feature = "reqwest")]
pub use client_config::default_client_builder;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fmt::Debug};

pub type Data = Vec<u8>;

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
