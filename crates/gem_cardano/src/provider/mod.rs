pub mod balances;
pub mod balances_mapper;
pub mod preload;
pub mod preload_mapper;
pub mod state;
pub mod token;
pub mod transactions;
pub mod transactions_mapper;

// Empty ChainAccount implementation
use async_trait::async_trait;
use chain_traits::ChainAccount;
use gem_client::Client;
use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainAccount for CardanoClient<C> {}
