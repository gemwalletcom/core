use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainState;

use crate::rpc::client::EthereumClient;
use gem_client::Client;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainState for EthereumClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        let chain_id = EthereumClient::get_chain_id(self).await?;
        Ok(u64::from_str_radix(chain_id.trim_start_matches("0x"), 16)?.to_string())
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        let block_number = EthereumClient::get_block_number(self).await?;
        Ok(u64::from_str_radix(block_number.trim_start_matches("0x"), 16)?)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{create_ethereum_test_client, create_smartchain_test_client};
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_ethereum_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let chain_id = ChainState::get_chain_id(&client).await?;

        println!("Ethereum Chain ID: {}", chain_id);

        assert_eq!(chain_id, "1");

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let block_number = ChainState::get_block_latest_number(&client).await?;

        println!("Ethereum Latest Block: {}", block_number);

        assert!(block_number > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let chain_id = ChainState::get_chain_id(&client).await?;

        println!("SmartChain Chain ID: {}", chain_id);

        assert_eq!(chain_id, "56");

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let block_number = ChainState::get_block_latest_number(&client).await?;

        println!("SmartChain Latest Block: {}", block_number);

        assert!(block_number > 0);

        Ok(())
    }
}
