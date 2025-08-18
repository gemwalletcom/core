pub mod alien_client;
pub mod alien_provider;

use primitives::Chain;
use std::sync::Arc;

pub use alien_client::AlienClient;
pub use alien_provider::{mime, mock, target::X_CACHE_TTL, AlienError, AlienHttpMethod, AlienProvider, AlienTarget};
pub use gem_jsonrpc::client::JsonRpcClient;
pub use gem_jsonrpc::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcResult};

// Helper function to create JsonRpcClient with chain-based endpoint resolution
pub fn jsonrpc_client_with_chain(provider: Arc<dyn AlienProvider>, chain: Chain) -> JsonRpcClient<AlienClient> {
    let endpoint = provider.get_endpoint(chain).expect("Failed to get endpoint for chain");
    let alien_client = AlienClient::new(endpoint.clone(), provider);
    JsonRpcClient::new(endpoint, alien_client)
}
