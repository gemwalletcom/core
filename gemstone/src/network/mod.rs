pub mod alien_client;
pub mod alien_provider;

use primitives::Chain;
use std::sync::Arc;

pub use alien_client::AlienClient;
pub use alien_provider::{AlienError, AlienHttpMethod, AlienProvider, AlienSigner, AlienTarget, X_CACHE_TTL};
pub use alien_provider::{mime, mock, target};
pub use gem_jsonrpc::client::JsonRpcClient;
pub use gem_jsonrpc::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcResult, JsonRpcResults};

pub fn jsonrpc_client_with_chain(provider: Arc<dyn AlienProvider>, chain: Chain) -> JsonRpcClient<AlienClient> {
    let endpoint = provider.get_endpoint(chain).expect("Failed to get endpoint for chain");
    let alien_client = AlienClient::new(endpoint, provider);
    JsonRpcClient::new(alien_client)
}
