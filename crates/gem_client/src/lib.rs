mod content_type;
mod types;

#[cfg(feature = "reqwest")]
mod reqwest_client;

#[cfg(feature = "reqwest")]
pub mod retry;

pub use content_type::ContentType;
pub use types::ClientError;

#[cfg(feature = "reqwest")]
pub use reqwest_client::ReqwestClient;

#[cfg(feature = "reqwest")]
pub use retry::{aggressive_retry_policy, retry_policy, standard_retry_policy};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fmt::Debug};

pub type Data = Vec<u8>;

#[async_trait]
pub trait Client: Send + Sync + Debug {
    async fn get<R>(&self, path: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned;
    async fn post<T, R>(&self, path: &str, body: &T, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned;
}
