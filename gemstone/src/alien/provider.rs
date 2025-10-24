use super::{AlienError, AlienResponse, AlienTarget};

use async_trait::async_trait;
use gem_jsonrpc::RpcProvider as GenericRpcProvider;
use primitives::Chain;
use std::{fmt::Debug, sync::Arc};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait AlienProvider: Send + Sync + Debug {
    async fn request(&self, target: AlienTarget) -> Result<AlienResponse, AlienError>;
    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError>;
}

#[derive(Debug)]
pub struct AlienProviderWrapper {
    pub provider: Arc<dyn AlienProvider>,
}

#[async_trait]
impl GenericRpcProvider for AlienProviderWrapper {
    type Error = AlienError;

    async fn request(&self, target: AlienTarget) -> Result<AlienResponse, Self::Error> {
        self.provider.request(target).await
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error> {
        self.provider.get_endpoint(chain)
    }
}
