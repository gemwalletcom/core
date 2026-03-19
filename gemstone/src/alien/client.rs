use super::AlienProvider;
use super::provider::AlienProviderWrapper;
use std::sync::Arc;
use swapper::RpcClient;

pub type AlienClient = RpcClient;

pub fn new_alien_client(base_url: String, provider: Arc<dyn AlienProvider>) -> AlienClient {
    RpcClient::new(base_url, Arc::new(AlienProviderWrapper::new(provider)))
}
