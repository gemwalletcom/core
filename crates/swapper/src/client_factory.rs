use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::alien::{self, RpcClient, RpcProvider};
use gem_jsonrpc::client::JsonRpcClient;
use primitives::{Chain, EVMChain};
use std::sync::Arc;

use crate::SwapperError;

pub fn create_client_with_chain(provider: Arc<dyn RpcProvider>, chain: Chain) -> JsonRpcClient<RpcClient> {
    alien::create_client(provider, chain).expect("failed to create client for chain")
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
    use gem_solana::{jsonrpc::SolanaRpc, models::blockhash::SolanaBlockhashResult};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_solana_json_rpc() -> Result<(), String> {
        let rpc_client = create_client_with_chain(Arc::new(NativeProvider::default()), Chain::Solana);
        let response: SolanaBlockhashResult = rpc_client.request(SolanaRpc::GetLatestBlockhash).await.map_err(|e| e.to_string())?;
        let recent_blockhash = response.value.blockhash;

        println!("recent_blockhash: {}", recent_blockhash);

        let blockhash = bs58::decode(recent_blockhash).into_vec().map_err(|_| "Failed to decode blockhash".to_string())?;

        let blockhash_array: [u8; 32] = blockhash.try_into().map_err(|_| "Failed to convert blockhash to array".to_string())?;

        assert_eq!(blockhash_array.len(), 32);

        Ok(())
    }
}
