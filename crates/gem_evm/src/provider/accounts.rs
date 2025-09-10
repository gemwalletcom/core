#[cfg(feature = "rpc")]
use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainProvider, ChainTraits};
use gem_client::Client;
use primitives::Chain;

use crate::rpc::client::EthereumClient;

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainTraits for EthereumClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainProvider for EthereumClient<C> {
    fn get_chain(&self) -> Chain {
        self.get_chain()
    }
}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainAccount for EthereumClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainPerpetual for EthereumClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainAddressStatus for EthereumClient<C> {}
