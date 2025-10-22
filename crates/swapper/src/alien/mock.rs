use async_trait::async_trait;
use std::{
    fmt::{self, Debug},
    time::Duration,
};

use super::{AlienError, Target};
use gem_jsonrpc::{RpcProvider as GenericRpcProvider, RpcResponse};
use primitives::Chain;

#[allow(unused)]
pub struct MockFn(pub Box<dyn Fn(Target) -> String + Send + Sync>);

impl fmt::Debug for MockFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MockFn").finish()
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct ProviderMock {
    pub response: MockFn,
    pub timeout: Duration,
}

#[allow(unused)]
impl ProviderMock {
    pub fn new(string: String) -> Self {
        Self {
            response: MockFn(Box::new(move |_| string.clone())),
            timeout: Duration::from_millis(100),
        }
    }
}

#[async_trait]
impl GenericRpcProvider for ProviderMock {
    type Error = AlienError;

    async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error> {
        Ok(RpcResponse {
            status: None,
            data: (self.response.0)(target).into_bytes(),
        })
    }

    fn get_endpoint(&self, _chain: Chain) -> Result<String, Self::Error> {
        Ok(String::from("http://localhost:8080"))
    }
}
