pub use alien_provider::{jsonrpc::JsonRpcClient, target::X_CACHE_TTL, AlienError, AlienHeader, AlienHttpMethod, AlienProvider, AlienTarget};
pub use primitives::jsonrpc_types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcResult};

#[uniffi::export]
fn alien_method_to_string(method: AlienHttpMethod) -> String {
    method.into()
}
