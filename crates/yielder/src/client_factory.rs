use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::alien::{self, RpcClient, RpcProvider};
use gem_jsonrpc::client::JsonRpcClient;
use primitives::{Chain, EVMChain};
use std::sync::Arc;

use crate::YielderError;

pub fn create_client(provider: Arc<dyn RpcProvider>, chain: Chain) -> Result<JsonRpcClient<RpcClient>, YielderError> {
    alien::create_client(provider, chain).map_err(|_| YielderError::NotSupportedChain)
}

pub fn create_eth_client(provider: Arc<dyn RpcProvider>, chain: Chain) -> Result<EthereumClient<RpcClient>, YielderError> {
    let evm_chain = EVMChain::from_chain(chain).ok_or(YielderError::NotSupportedChain)?;
    let client = create_client(provider, chain)?;
    Ok(EthereumClient::new(client, evm_chain))
}
