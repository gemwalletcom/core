use super::AlienError;
use super::{AlienTarget, JsonRpcRequest, JsonRpcResult};
use async_trait::async_trait;
use std::fmt::Debug;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait AlienProvider: Send + Sync + Debug {
    async fn request(&self, target: AlienTarget) -> Result<Vec<u8>, AlienError>;
    async fn jsonrpc_call(&self, requests: Vec<JsonRpcRequest>, chain: primitives::Chain) -> Result<Vec<JsonRpcResult>, AlienError>;
}
