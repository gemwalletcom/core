mod asset;
mod chain;
mod client;
mod model;
mod provider;
mod quote_data_mapper;

use std::sync::Arc;

use crate::alien::RpcProvider;
use gem_client::Client;

use super::{ProviderType, SwapperProvider};

const RELAY_API_URL: &str = "https://api.relay.link";
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
