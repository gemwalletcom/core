pub mod balances;
pub mod balances_mapper;
pub mod perpetual;
pub mod perpetual_mapper;
pub mod staking;
pub mod staking_mapper;
pub mod state;
pub mod token;
pub mod transactions;
pub mod transactions_mapper;

// Empty ChainAccount implementation
use crate::rpc::client::HyperCoreClient;
use async_trait::async_trait;
use chain_traits::ChainAccount;
use gem_client::Client;

#[async_trait]
impl<C: Client> ChainAccount for HyperCoreClient<C> {}
