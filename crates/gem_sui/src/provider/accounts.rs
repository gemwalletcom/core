#[cfg(feature = "rpc")]
use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainProvider, ChainTraits};
#[cfg(feature = "rpc")]
use gem_client::Client;
use primitives::Chain;

use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainTraits for SuiClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainProvider for SuiClient<C> {
    fn get_chain(&self) -> Chain {
        self.chain
    }
}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainAccount for SuiClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainPerpetual for SuiClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainAddressStatus for SuiClient<C> {}
