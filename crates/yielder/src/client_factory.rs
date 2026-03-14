use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::{RpcClient, RpcClientError, RpcProvider};
use primitives::{Chain, EVMChain};
use std::sync::Arc;

use crate::error::YielderError;

pub fn create_eth_client<E: RpcClientError + Clone + 'static>(provider: Arc<dyn RpcProvider<Error = E>>, chain: Chain) -> Result<EthereumClient<RpcClient<E>>, YielderError> {
    let evm_chain = EVMChain::from_chain(chain).ok_or_else(|| YielderError::unsupported_chain(&chain))?;
    let endpoint = provider.get_endpoint(chain).map_err(|e| YielderError::NetworkError(e.to_string()))?;
    let client = RpcClient::new(endpoint, provider);
    Ok(EthereumClient::new(JsonRpcClient::new(client), evm_chain))
}
