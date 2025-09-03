use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainBalances;
use primitives::{AssetBalance, EVMChain};

use crate::provider::balances_mapper::{map_balance_coin, map_balance_tokens};
use crate::rpc::client::EthereumClient;
use gem_client::Client;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainBalances for EthereumClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        map_balance_coin(self.get_eth_balance(&address).await?, self.get_chain())
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balance_results = self.batch_token_balance_calls(&address, &token_ids).await?;
        map_balance_tokens(balance_results, token_ids, self.get_chain())
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        match self.chain {
            EVMChain::SmartChain => self.get_smartchain_staking_balance(&address).await,
            _ => Ok(None),
        }
    }

    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        unimplemented!("get_assets_balances")
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{
        create_arbitrum_test_client, create_ethereum_test_client, create_smartchain_test_client, TEST_ADDRESS, TEST_SMARTCHAIN_STAKING_ADDRESS,
        TOKEN_DAI_ADDRESS, TOKEN_USDC_ADDRESS,
    };
    use chain_traits::ChainBalances;
    use num_bigint::BigUint;
    use primitives::Chain;

    #[tokio::test]
    async fn test_ethereum_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;

        println!("Ethereum ETH Balance: {:?}", balance.balance.available);

        assert_eq!(balance.asset_id.chain, Chain::Ethereum);
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));

        Ok(())
    }

    #[tokio::test]
    async fn test_arbitrum_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_arbitrum_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;

        println!("Arbitrum ETH Balance: {:?}", balance.balance.available);

        assert_eq!(balance.asset_id.chain, Chain::Arbitrum);
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;

        println!("Smartchain BNB Balance: {:?}", balance.balance.available);

        assert_eq!(balance.asset_id.chain, Chain::SmartChain);
        assert!(balance.balance.available > BigUint::from(0u32));

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let balance = client.get_balance_staking(TEST_SMARTCHAIN_STAKING_ADDRESS.to_string()).await?.unwrap();

        println!("Smartchain BNB Balance: {:?}", balance);

        assert!(balance.balance.staked > BigUint::from(1_000_000_000_000_000_000u64));

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let token_ids = vec![TOKEN_USDC_ADDRESS.to_string(), TOKEN_DAI_ADDRESS.to_string()];

        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids).await?;

        println!("USDC Balance: {:?}", balances);

        assert_eq!(balances.len(), 2);

        assert_eq!(balances[0].asset_id.chain, Chain::Ethereum);
        assert_eq!(balances[0].asset_id.token_id, Some(TOKEN_USDC_ADDRESS.to_string()));
        assert!(balances[0].balance.available > BigUint::from(0u32));

        assert_eq!(balances[1].asset_id.chain, Chain::Ethereum);
        assert_eq!(balances[1].asset_id.token_id, Some(TOKEN_DAI_ADDRESS.to_string()));
        assert!(balances[1].balance.available > BigUint::from(0u32));

        Ok(())
    }
}
