use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainBalances;
use primitives::AssetBalance;

use crate::provider::balances_mapper::map_balance_coin;
use crate::rpc::client::EthereumClient;
use gem_client::Client;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainBalances for EthereumClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        map_balance_coin(self.get_eth_balance(&address).await?, self.get_chain())
    }

    async fn get_balance_tokens(&self, _address: String, _token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        unimplemented!("get_balance_tokens")
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        unimplemented!("get_balance_staking")
    }

    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        unimplemented!("get_assets_balances")
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_arbitrum_test_client, create_ethereum_test_client, create_smartchain_test_client, TEST_ADDRESS};
    use primitives::Chain;

    #[tokio::test]
    async fn test_ethereum_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;

        println!("Ethereum ETH Balance: {:?}", balance.balance.available);

        assert_eq!(balance.asset_id.chain, Chain::Ethereum);
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));

        Ok(())
    }

    #[tokio::test]
    async fn test_arbitrum_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_arbitrum_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;

        println!("Arbitrum ETH Balance: {:?}", balance.balance.available);

        assert_eq!(balance.asset_id.chain, Chain::Arbitrum);
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;

        println!("Smartchain BNB Balance: {:?}", balance.balance.available);

        assert_eq!(balance.asset_id.chain, Chain::SmartChain);
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));

        Ok(())
    }
}
