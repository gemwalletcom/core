use chain_traits::ChainTraits;
use gem_client::Client;

use crate::rpc::client::AptosClient;

pub mod balances;
pub mod balances_mapper;
pub mod preload;
pub mod preload_mapper;
pub mod state;
pub mod state_mapper;
pub mod token;
pub mod token_mapper;
pub mod transactions;
pub mod transactions_mapper;

impl<C: Client> ChainTraits for AptosClient<C> {}
