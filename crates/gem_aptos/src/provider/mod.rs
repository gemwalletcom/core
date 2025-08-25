use chain_traits::{ChainProvider, ChainTraits};
use gem_client::Client;
use primitives::Chain;

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

impl<C: Client> ChainProvider for AptosClient<C> {
    fn get_chain(&self) -> Chain {
        self.get_chain()
    }
}
