use crate::alien::{RpcClient, RpcProvider};
use std::sync::Arc;

use super::provider::ProxyProvider;

pub fn new_stonfi_v2(rpc_provider: Arc<dyn RpcProvider>) -> ProxyProvider<RpcClient> {
    ProxyProvider::new_stonfi_v2(rpc_provider)
}

pub fn new_orca(rpc_provider: Arc<dyn RpcProvider>) -> ProxyProvider<RpcClient> {
    ProxyProvider::new_orca(rpc_provider)
}

pub fn new_cetus_aggregator(rpc_provider: Arc<dyn RpcProvider>) -> ProxyProvider<RpcClient> {
    ProxyProvider::new_cetus_aggregator(rpc_provider)
}

pub fn new_mayan(rpc_provider: Arc<dyn RpcProvider>) -> ProxyProvider<RpcClient> {
    ProxyProvider::new_mayan(rpc_provider)
}

pub fn new_relay(rpc_provider: Arc<dyn RpcProvider>) -> ProxyProvider<RpcClient> {
    ProxyProvider::new_relay(rpc_provider)
}
