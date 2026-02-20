use crate::{Client, ClientError};
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    sync::Arc,
};

type GetHandler = Arc<dyn Fn(&str) -> Result<Vec<u8>, ClientError> + Send + Sync>;
type PostHandler = Arc<dyn Fn(&str, &[u8]) -> Result<Vec<u8>, ClientError> + Send + Sync>;

#[derive(Clone, Default)]
pub struct MockClient {
    get_handler: Option<GetHandler>,
    post_handler: Option<PostHandler>,
}

impl MockClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_get<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) -> Result<Vec<u8>, ClientError> + Send + Sync + 'static,
    {
        self.get_handler = Some(Arc::new(handler));
        self
    }

    pub fn with_post<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, &[u8]) -> Result<Vec<u8>, ClientError> + Send + Sync + 'static,
    {
        self.post_handler = Some(Arc::new(handler));
        self
    }
}

impl Debug for MockClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockClient").finish()
    }
}

#[async_trait]
impl Client for MockClient {
    async fn get_with<R>(&self, path: &str, _query: &[(String, String)], _headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let handler = self.get_handler.as_ref().ok_or(ClientError::Http { status: 404, body: vec![] })?;
        let bytes = handler(path)?;
        serde_json::from_slice(&bytes).map_err(|e| ClientError::Serialization(e.to_string()))
    }

    async fn post_with<T, R>(&self, path: &str, body: &T, _headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let handler = self.post_handler.as_ref().ok_or(ClientError::Http { status: 404, body: vec![] })?;
        let body_bytes = serde_json::to_vec(body).map_err(|e| ClientError::Serialization(e.to_string()))?;
        let bytes = handler(path, &body_bytes)?;
        serde_json::from_slice(&bytes).map_err(|e| ClientError::Serialization(e.to_string()))
    }
}
