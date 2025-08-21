pub mod balances;
pub mod balances_mapper;
pub mod preload;
pub mod preload_mapper;
pub mod state;
pub mod transactions;

// Empty ChainAccount implementation
use async_trait::async_trait;
use chain_traits::ChainAccount;
use gem_client::Client;
use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainAccount for BitcoinClient<C> {}
