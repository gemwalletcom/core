use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use number_formatter::BigNumberFormatter;
use primitives::AssetBalance;

use super::balances_mapper::{map_balance_coin, map_balance_staking};
use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainBalances for HyperCoreClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let total = self
            .get_spot_balances(&address)
            .await?
            .balances
            .into_iter()
            .find(|x| x.token == 150)
            .ok_or("not found")?
            .total;
        let available: String = BigNumberFormatter::value_from_amount(&total, 18)?;
        Ok(map_balance_coin(available, self.chain))
    }

    async fn get_balance_tokens(&self, _address: String, _token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balance = self.get_stake_balance(&address).await?;
        Ok(Some(map_balance_staking(&balance, self.chain)?))
    }

    async fn get_balance_assets(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, create_hypercore_test_client};
    use chain_traits::ChainBalances;
    use num_bigint::BigUint;

    #[tokio::test]
    async fn test_hypercore_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;

        println!("Hypercore coin balance: {:?} {}", balance.balance.available, balance.asset_id);

        assert!(balance.balance.available >= BigUint::from(0u64));
        assert_eq!(balance.asset_id.chain, primitives::Chain::HyperCore);
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let address = TEST_ADDRESS.to_string();
        let token_balances = client.get_balance_tokens(address, vec!["USDC".to_string()]).await?;

        println!("Hypercore token balances count: {}", token_balances.len());

        assert_eq!(token_balances.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_staking(address).await?.ok_or("not found")?;

        println!("Hypercore staking balance: {:?}", balance.balance.staked);

        assert!(balance.balance.staked >= BigUint::from(0u64));
        assert_eq!(balance.asset_id.chain, primitives::Chain::HyperCore);
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let address = TEST_ADDRESS.to_string();
        let assets = client.get_balance_assets(address).await?;

        assert_eq!(assets.len(), 0);
        Ok(())
    }
}
