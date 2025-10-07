use super::provider::AlienProviderWrapper;
use super::AlienProvider;
use gem_swapper::{RpcClient, RpcProvider};
use std::sync::Arc;

pub type AlienClient = RpcClient;

pub fn new_alien_client(base_url: String, provider: Arc<dyn AlienProvider>) -> AlienClient {
    let wrapper: Arc<dyn RpcProvider> = Arc::new(AlienProviderWrapper { provider });
    RpcClient::new(base_url, wrapper)
}
