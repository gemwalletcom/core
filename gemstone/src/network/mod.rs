use async_trait::async_trait;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

#[derive(Debug, uniffi::Error, thiserror::Error)]
pub enum AlienError {
    #[error("URL is invalid: {url}")]
    InvalidURL { url: String },
}

#[derive(Debug, uniffi::Record)]
pub struct AlienTarget {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait AlienProvider: Send + Sync + Debug {
    async fn request(&self, target: AlienTarget) -> Result<Vec<u8>, AlienError>;
}

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

    pub async fn teleport(&self, target: AlienTarget) -> Result<Vec<u8>, AlienError> {
        self.provider.request(target).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::future::{pending, timeout};
    use async_trait::async_trait;
    use std::time::Duration;

    #[derive(Debug)]
    pub struct AlienProviderMock {
        response: String,
    }

    #[async_trait]
    impl AlienProvider for AlienProviderMock {
        async fn request(&self, _target: AlienTarget) -> Result<Vec<u8>, AlienError> {
            let never = pending::<()>();
            let _ = timeout(Duration::from_millis(200), never).await;
            Ok(self.response.as_bytes().to_vec())
        }
    }

    #[tokio::test]
    async fn test_mock_call() {
        let mock = AlienProviderMock {
            response: String::from("Hello"),
        };
        let warp = AlienProviderWarp {
            provider: std::sync::Arc::new(mock),
        };
        let target = AlienTarget {
            url: String::from("https://example.com"),
            method: String::from("GET"),
            headers: None,
            body: None,
        };
        let bytes = warp.teleport(target).await.unwrap();
        let string = String::from_utf8(bytes).unwrap();

        assert_eq!("Hello", string);
    }
}
