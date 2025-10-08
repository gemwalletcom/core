use super::AlienProvider;
use super::provider::AlienProviderWrapper;
use std::sync::Arc;
use swapper::{RpcClient, RpcProvider};

pub type AlienClient = RpcClient;

pub fn new_alien_client(base_url: String, provider: Arc<dyn AlienProvider>) -> AlienClient {
    let wrapper: Arc<dyn RpcProvider> = Arc::new(AlienProviderWrapper { provider });
    RpcClient::new(base_url, wrapper)
}
