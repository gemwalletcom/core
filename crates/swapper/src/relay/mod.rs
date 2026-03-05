mod asset;
mod chain;
mod client;
mod mapper;
mod model;
mod provider;
#[cfg(test)]
mod testkit;

use std::sync::Arc;

use crate::alien::RpcProvider;
use gem_client::Client;

use super::{ProviderType, SwapperProvider};

const DEFAULT_SWAP_GAS_LIMIT: u64 = 150_000;

#[derive(Debug)]
pub struct Relay<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
    client: client::RelayClient<C>,
}

impl<C> Relay<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub fn with_client(client: client::RelayClient<C>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Relay),
            rpc_provider,
            client,
        }
    }
}
