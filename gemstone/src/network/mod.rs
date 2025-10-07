pub mod tron;
use crate::alien::{AlienClient, AlienProvider, new_alien_client};
use primitives::Chain;
use std::sync::Arc;

pub use gem_jsonrpc::client::JsonRpcClient;
pub use gem_jsonrpc::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcResult, JsonRpcResults};
pub use tron::tron_client;

pub fn jsonrpc_client_with_chain(provider: Arc<dyn AlienProvider>, chain: Chain) -> JsonRpcClient<AlienClient> {
    let endpoint = provider.get_endpoint(chain).expect("Failed to get endpoint for chain");
    let alien_client = new_alien_client(endpoint, provider);
    JsonRpcClient::new(alien_client)
}
