use super::{AlienError, AlienProvider, AlienTarget, Data};
use async_trait::async_trait;
use primitives::Chain;
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

use std::fmt;

pub struct MockFn(pub Box<dyn Fn(AlienTarget) -> String + Send + Sync>);

impl fmt::Debug for MockFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MockFn").finish()
    }
}

#[derive(Debug)]
pub struct AlienProviderMock {
    pub response: MockFn,
    pub timeout: Duration,
}

impl AlienProviderMock {
    pub fn new(string: String) -> Self {
        Self {
            response: MockFn(Box::new(move |_| string.clone())),
            timeout: Duration::from_millis(100),
        }
    }
}

#[async_trait]
impl AlienProvider for AlienProviderMock {
    async fn request(&self, target: AlienTarget) -> Result<Data, AlienError> {
        let responses = self.batch_request(vec![target]).await;
        responses.map(|responses| responses.first().unwrap().clone())
    }

    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
        targets.iter().map(|target| Ok(self.response.0(target.clone()).into_bytes())).collect()
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
        let target = AlienTarget {
            url: String::from("https://example.com"),
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        };
        let mock = AlienProviderMock {
            response: MockFn(Box::new(|_| String::from("Hello"))),
            timeout: Duration::from_millis(100),
        };
        let warp = AlienProviderWarp {
            provider: std::sync::Arc::new(mock),
        };

        let data_vec = warp.teleport(vec![target]).await.unwrap();
        let bytes = data_vec.first().unwrap();
        let string = String::from_utf8(bytes.clone()).unwrap();

        assert_eq!("Hello", string);
    }
}
