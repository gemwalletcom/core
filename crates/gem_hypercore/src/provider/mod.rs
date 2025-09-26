use async_trait::async_trait;
use chain_traits::ChainAccount;
use gem_client::Client;

pub mod balances;
pub mod balances_mapper;
pub mod perpetual;
pub mod perpetual_mapper;
pub mod preload;
pub mod preload_cache;
pub mod preload_mapper;
pub mod staking;
pub mod staking_mapper;
pub mod state;
pub mod testkit;
pub mod token;
pub mod transactions;
pub mod transactions_mapper;
pub mod transaction_state;

use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainAccount for HyperCoreClient<C> {}
