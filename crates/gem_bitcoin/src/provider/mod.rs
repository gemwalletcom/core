pub mod balances;
pub mod balances_mapper;
pub mod preload;
pub mod preload_mapper;
pub mod request_classifier;
pub mod state;
pub mod state_mapper;
pub mod testkit;
pub mod transaction_broadcast;
pub mod transaction_broadcast_mapper;
pub mod transaction_state;
pub mod transactions;
pub mod transactions_mapper;

pub struct BroadcastProvider;

pub use transactions_mapper::map_transaction;

// Empty ChainAccount implementation
use crate::rpc::client::BitcoinClient;
use async_trait::async_trait;
use chain_traits::ChainAccount;
use gem_client::Client;

#[async_trait]
impl<C: Client> ChainAccount for BitcoinClient<C> {}
