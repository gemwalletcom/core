pub mod balances;
pub mod balances_mapper;
pub mod preload;
pub mod preload_mapper;
pub mod state;
#[cfg(all(test, feature = "integration_tests"))]
pub mod testkit;
pub mod token;
pub mod transactions;
pub mod transactions_mapper;

pub use transactions_mapper::map_transaction;

// Empty ChainAccount implementation
use crate::rpc::client::CardanoClient;
use async_trait::async_trait;
use chain_traits::ChainAccount;
use gem_client::Client;

#[async_trait]
impl<C: Client> ChainAccount for CardanoClient<C> {}
