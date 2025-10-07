use super::{AlienClient, AlienError, AlienProvider};

use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use primitives::Chain;
use std::{fmt::Debug, sync::Arc};

pub trait JsonRpcClientFactory<C>: Send + Sync
where
    C: Client + Clone + Debug + Send + Sync + 'static,
{
    fn client_for(&self, chain: Chain) -> Result<JsonRpcClient<C>, AlienError>;
}

#[derive(Clone)]
pub struct AlienEvmRpcFactory {
    provider: Arc<dyn AlienProvider>,
}

impl AlienEvmRpcFactory {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }
}

impl JsonRpcClientFactory<AlienClient> for AlienEvmRpcFactory {
    fn client_for(&self, chain: Chain) -> Result<JsonRpcClient<AlienClient>, AlienError> {
        let endpoint = self.provider.get_endpoint(chain)?;
        let client = AlienClient::new(endpoint, self.provider.clone());
        Ok(JsonRpcClient::new(client))
    }
}

pub fn jsonrpc_client_with_chain(provider: Arc<dyn AlienProvider>, chain: Chain) -> JsonRpcClient<AlienClient> {
    let endpoint = provider.get_endpoint(chain).expect("Failed to get endpoint for chain");
    let alien_client = AlienClient::new(endpoint, provider);
    JsonRpcClient::new(alien_client)
}
