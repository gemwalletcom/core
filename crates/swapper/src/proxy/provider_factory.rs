use std::sync::Arc;

use super::provider::ProxyProvider;
use crate::alien::{RpcClient, RpcProvider};

pub fn new_okx(rpc_provider: Arc<dyn RpcProvider>) -> ProxyProvider<RpcClient> {
    ProxyProvider::new_okx(rpc_provider)
}
