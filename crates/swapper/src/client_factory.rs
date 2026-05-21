use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::alien::{self, RpcClient, RpcProvider};
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::grpc::AlienGrpcTransport;
use gem_sui::rpc::client::SuiClient;
use primitives::{Chain, EVMChain};
use std::sync::Arc;

use crate::SwapperError;

pub fn create_client_with_chain(provider: Arc<dyn RpcProvider>, chain: Chain) -> JsonRpcClient<RpcClient> {
    alien::create_client(provider, chain).expect("failed to create client for chain")
}

pub fn create_sui_client(provider: Arc<dyn RpcProvider>) -> Result<SuiClient, SwapperError> {
    let endpoint = provider.get_endpoint(Chain::Sui).map_err(|_| SwapperError::NotSupportedChain)?;
    Ok(SuiClient::new_with_transport(endpoint, Arc::new(AlienGrpcTransport::new(provider))))
}

pub fn create_eth_client(provider: Arc<dyn RpcProvider>, chain: Chain) -> Result<EthereumClient<RpcClient>, SwapperError> {
    let evm_chain = EVMChain::from_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
    let client = alien::create_client(provider, chain).map_err(|_| SwapperError::NotSupportedChain)?;
    Ok(EthereumClient::new(client, evm_chain))
}

#[cfg(all(test, feature = "reqwest_provider", feature = "swap_integration_tests"))]
mod tests {
    use super::*;
    use crate::NativeProvider;
    use gem_solana::{jsonrpc::SolanaRpc, models::blockhash::SolanaBlockhashResult, try_decode_blockhash};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_solana_json_rpc() -> Result<(), String> {
        let rpc_client = create_client_with_chain(Arc::new(NativeProvider::default()), Chain::Solana);
        let response: SolanaBlockhashResult = rpc_client.request(SolanaRpc::GetLatestBlockhash).await.map_err(|e| e.to_string())?;
        let recent_blockhash = response.value.blockhash;

        println!("recent_blockhash: {}", recent_blockhash);

        let blockhash_array = try_decode_blockhash(&recent_blockhash).ok_or_else(|| "Invalid Solana blockhash".to_string())?;

        assert_eq!(blockhash_array.len(), 32);

        Ok(())
    }
}
