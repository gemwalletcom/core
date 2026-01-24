mod asset;
mod bigint;
mod chain;
mod client;
mod constants;
mod memo;
pub(crate) mod model;
mod provider;
mod quote_data_mapper;

use primitives::Chain;
use std::sync::Arc;

use crate::alien::RpcProvider;
use gem_client::Client;

use super::{ProviderType, SwapperProvider};

const QUOTE_MINIMUM: i64 = 0;
const QUOTE_INTERVAL: i64 = 1;
const QUOTE_QUANTITY: i64 = 0;
const OUTBOUND_DELAY_SECONDS: u32 = 60;

// FIXME: estimate gas limit with memo x bytes
const DEFAULT_DEPOSIT_GAS_LIMIT: u64 = 90_000;

#[derive(Debug)]
pub struct ThorChain<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub provider: ProviderType,
    pub rpc_provider: Arc<dyn RpcProvider>,
    pub(crate) swap_client: client::ThorChainSwapClient<C>,
}

impl<C> ThorChain<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub fn with_client(swap_client: client::ThorChainSwapClient<C>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Thorchain),
            rpc_provider,
            swap_client,
        }
    }

    fn get_eta_in_seconds(&self, destination_chain: Chain, total_swap_seconds: Option<u32>) -> u32 {
        destination_chain.block_time() / 1000 + OUTBOUND_DELAY_SECONDS + total_swap_seconds.unwrap_or(0)
    }
}
