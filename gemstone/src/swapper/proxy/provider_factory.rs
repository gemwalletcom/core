use crate::network::{AlienClient, AlienProvider};
use std::sync::Arc;

use super::provider::ProxyProvider;

pub fn new_stonfi_v2(rpc_provider: Arc<dyn AlienProvider>) -> ProxyProvider<AlienClient> {
    ProxyProvider::new_stonfi_v2(rpc_provider)
}

pub fn new_symbiosis(rpc_provider: Arc<dyn AlienProvider>) -> ProxyProvider<AlienClient> {
    ProxyProvider::new_symbiosis(rpc_provider)
}

pub fn new_cetus_aggregator(rpc_provider: Arc<dyn AlienProvider>) -> ProxyProvider<AlienClient> {
    ProxyProvider::new_cetus_aggregator(rpc_provider)
}

pub fn new_mayan(rpc_provider: Arc<dyn AlienProvider>) -> ProxyProvider<AlienClient> {
    ProxyProvider::new_mayan(rpc_provider)
}

pub fn new_relay(rpc_provider: Arc<dyn AlienProvider>) -> ProxyProvider<AlienClient> {
    ProxyProvider::new_relay(rpc_provider)
}
