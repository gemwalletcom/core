mod types;

#[cfg(feature = "reqwest")]
mod reqwest_client;

pub use types::ClientError;

#[cfg(feature = "reqwest")]
pub use reqwest_client::ReqwestClient;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fmt::Debug};

pub type Data = Vec<u8>;

#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    ApplicationJson,
    TextPlain,
}

impl ContentType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ContentType::ApplicationJson => "application/json",
            ContentType::TextPlain => "text/plain",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "application/json" => Some(ContentType::ApplicationJson),
            "text/plain" => Some(ContentType::TextPlain),
            _ => None,
        }
    }
}

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
