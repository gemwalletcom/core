use super::{AlienError, AlienProvider, AlienTarget, provider::Data};
use async_trait::async_trait;
use gem_swapper::RpcProvider;
use primitives::Chain;

pub use gem_swapper::alien::reqwest_provider::NativeProvider;

#[async_trait]
impl AlienProvider for NativeProvider {
    async fn request(&self, target: AlienTarget) -> Result<Data, AlienError> {
        RpcProvider::request(self, target).await
    }

    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
        RpcProvider::batch_request(self, targets).await
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        RpcProvider::get_endpoint(self, chain)
    }
}
