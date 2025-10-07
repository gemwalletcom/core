use super::{AlienError, AlienProvider, AlienTarget, provider::Data};
use async_trait::async_trait;
use gem_client::RpcProvider as GenericRpcProvider;
use primitives::Chain;

pub use gem_swapper::NativeProvider;

#[async_trait]
impl AlienProvider for NativeProvider {
    async fn request(&self, target: AlienTarget) -> Result<Data, AlienError> {
        <Self as GenericRpcProvider>::request(self, target).await
    }

    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
        <Self as GenericRpcProvider>::batch_request(self, targets).await
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        <Self as GenericRpcProvider>::get_endpoint(self, chain)
    }
}
