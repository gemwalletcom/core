use async_trait::async_trait;
use primitives::Chain;

use super::{AlienError, AlienProvider, AlienTarget, Data};
use std::{fmt::Debug, sync::Arc, time::Duration};

#[derive(Debug, uniffi::Object)]
pub struct AlienProviderWarp {
    pub provider: Arc<dyn AlienProvider>,
}

#[uniffi::export]
impl AlienProviderWarp {
    #[uniffi::constructor]
    fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn teleport(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
        self.provider.batch_request(targets).await
    }
}

#[derive(Debug)]
pub struct AlienProviderMock {
    pub response: String,
    pub timeout: Duration,
}

#[async_trait]
impl AlienProvider for AlienProviderMock {
    async fn request(&self, _target: AlienTarget) -> Result<Data, AlienError> {
        let responses = self.batch_request(vec![_target]).await;
        responses.map(|responses| responses.first().unwrap().clone())
    }

    async fn batch_request(&self, _targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
        Ok(vec![self.response.as_bytes().to_vec()])
    }

    fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
        Ok(String::from("http://localhost:8080"))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::network::{mock::*, target::*};
    use std::time::Duration;

    #[tokio::test]
    async fn test_mock_call() {
        let mock = AlienProviderMock {
            response: String::from("Hello"),
            timeout: Duration::from_millis(100),
        };
        let warp = AlienProviderWarp {
            provider: std::sync::Arc::new(mock),
        };
        let target = AlienTarget {
            url: String::from("https://example.com"),
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        };
        let data_vec = warp.teleport(vec![target]).await.unwrap();
        let bytes = data_vec.first().unwrap();
        let string = String::from_utf8(bytes.clone()).unwrap();

        assert_eq!("Hello", string);
    }
}
