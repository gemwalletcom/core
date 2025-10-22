use super::{AlienError, AlienProvider, AlienTarget};
use async_trait::async_trait;
use gem_jsonrpc::{RpcProvider as GenericRpcProvider, RpcResponse};
use primitives::Chain;

pub use swapper::NativeProvider;

#[async_trait]
impl AlienProvider for NativeProvider {
    async fn request(&self, target: AlienTarget) -> Result<RpcResponse, AlienError> {
        <Self as GenericRpcProvider>::request(self, target).await
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        <Self as GenericRpcProvider>::get_endpoint(self, chain)
    }
}
