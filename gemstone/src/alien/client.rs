use super::{AlienError, AlienProvider, AlienTarget};
use super::provider::AlienProviderWrapper;
use async_trait::async_trait;
use gem_client::{Client, ClientError};
use gem_swapper::{RpcClient, RpcProvider};
use primitives::Chain;
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct AlienClient {
    inner: RpcClient,
}

impl AlienClient {
    pub fn new(base_url: String, provider: Arc<dyn AlienProvider>) -> Self {
        let wrapper = Arc::new(AlienProviderWrapper { provider });
        Self {
            inner: RpcClient::new(base_url, wrapper),
        }
    }
}

#[async_trait]
impl Client for AlienClient {
    async fn get<R>(&self, path: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        self.inner.get(path).await
    }

    async fn get_with_headers<R>(&self, path: &str, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        self.inner.get_with_headers(path, headers).await
    }

    async fn post<T, R>(&self, path: &str, body: &T, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        self.inner.post(path, body, headers).await
    }
}

#[async_trait]
impl AlienProvider for AlienClient {
    async fn request(&self, target: AlienTarget) -> Result<Vec<u8>, AlienError> {
        self.inner.request(target).await
    }

    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Vec<u8>>, AlienError> {
        self.inner.batch_request(targets).await
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        self.inner.get_endpoint(chain)
    }
}
