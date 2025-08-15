mod types;

#[cfg(feature = "reqwest")]
mod reqwest_client;

pub use types::ClientError;

#[cfg(feature = "reqwest")]
pub use reqwest_client::ReqwestClient;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub type Data = Vec<u8>;

#[async_trait]
pub trait Client: Send + Sync + Debug {
    async fn get<R>(&self, path: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned;
    async fn post<T, R>(&self, path: &str, body: &T) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned;
}
