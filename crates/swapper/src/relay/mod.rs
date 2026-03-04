mod asset;
mod chain;
mod client;
mod cross_chain;
mod mapper;
mod model;
mod provider;
#[cfg(test)]
mod testkit;
mod tx_builder;

pub use cross_chain::{RELAY_CONTRACTS, RelayCrossChain};

use std::sync::Arc;

use crate::alien::RpcProvider;
use gem_client::Client;

use super::{ProviderType, SwapperProvider};

const DEFAULT_GAS_LIMIT: u64 = 750_000;

#[derive(Debug)]
pub struct Relay<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub provider: ProviderType,
    pub rpc_provider: Arc<dyn RpcProvider>,
    pub(crate) client: client::RelayClient<C>,
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
