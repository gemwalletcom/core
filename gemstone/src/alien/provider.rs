use super::{AlienError, AlienTarget};

use async_trait::async_trait;
use gem_swapper::RpcProvider;
use primitives::Chain;
use std::{fmt::Debug, sync::Arc};

pub type Data = Vec<u8>;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait AlienProvider: Send + Sync + Debug {
    async fn request(&self, target: AlienTarget) -> Result<Data, AlienError>;
    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError>;
    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError>;
}

#[derive(Debug)]
pub struct AlienProviderWrapper {
    pub provider: Arc<dyn AlienProvider>,
}

#[async_trait]
impl RpcProvider for AlienProviderWrapper {
    async fn request(&self, target: AlienTarget) -> Result<Data, AlienError> {
        self.provider.request(target).await
    }
    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
        self.provider.batch_request(targets).await
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, gem_swapper::AlienError> {
        self.provider.get_endpoint(chain)
    }
}
