use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_tron::rpc::{TronClient, trongrid::client::TronGridClient};
use primitives::{Chain, EVMChain};
use std::sync::Arc;

use crate::{
    SwapperError,
    alien::{AlienError, RpcClient, RpcProvider},
};

pub fn create_client_with_chain(provider: Arc<dyn RpcProvider>, chain: Chain) -> JsonRpcClient<RpcClient> {
    let endpoint = provider.get_endpoint(chain).expect("Failed to get endpoint for chain");
    let client = RpcClient::new(endpoint, provider);
    JsonRpcClient::new(client)
}

pub fn create_tron_client(provider: Arc<dyn RpcProvider>) -> Result<TronClient<RpcClient>, AlienError> {
    let endpoint = provider.get_endpoint(Chain::Tron)?;
    let tron_rpc_client = RpcClient::new(endpoint.clone(), provider.clone());
    let trongrid_client = TronGridClient::new(RpcClient::new(endpoint, provider), String::new());

    Ok(TronClient::new(tron_rpc_client, trongrid_client))
}

pub fn create_eth_client(provider: Arc<dyn RpcProvider>, chain: Chain) -> Result<EthereumClient<RpcClient>, SwapperError> {
    let evm_chain = EVMChain::from_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
    let client = create_client_with_chain(provider, chain);
    Ok(EthereumClient::new(client, evm_chain))
}
