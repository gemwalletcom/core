use std::{fmt::Debug, sync::Arc};

pub mod jsonrpc;
pub use jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcResult};
pub mod target;
pub use target::{AlienHttpMethod, AlienTarget};
pub mod provider;
pub use provider::{AlienProvider, Data};

#[derive(Debug, uniffi::Error, thiserror::Error)]
pub enum AlienError {
    #[error("Request is invalid: {msg}")]
    RequestError { msg: String },
    #[error("Request error: {msg}")]
    ResponseError { msg: String },
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

    pub async fn teleport(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
        self.provider.batch_request(targets).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::future::{pending, timeout};
    use async_trait::async_trait;
    use std::{time::Duration, vec};

    #[derive(Debug)]
    pub struct AlienProviderMock {
        response: String,
    }

    #[async_trait]
    impl AlienProvider for AlienProviderMock {
        async fn request(&self, _target: AlienTarget) -> Result<Data, AlienError> {
            let responses = self.batch_request(vec![_target]).await;
            responses.map(|responses| responses.first().unwrap().clone())
        }

        async fn batch_request(&self, _targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
            let never = pending::<()>();
            let _ = timeout(Duration::from_millis(200), never).await;
            Ok(vec![self.response.as_bytes().to_vec()])
        }

        fn get_endpoint(&self, _chain: primitives::Chain) -> Result<String, AlienError> {
            Ok(String::from("http://localhost:8080"))
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
