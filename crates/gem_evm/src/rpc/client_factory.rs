use gem_jsonrpc::alien::{self, RpcClient, RpcProvider};
use primitives::{Chain, EVMChain};
use std::sync::Arc;

use super::EthereumClient;

pub fn create_eth_client(provider: Arc<dyn RpcProvider>, chain: Chain) -> Option<EthereumClient<RpcClient>> {
    let evm_chain = EVMChain::from_chain(chain)?;
    let client = alien::create_client(provider, chain).ok()?;
    Some(EthereumClient::new(client, evm_chain))
}
