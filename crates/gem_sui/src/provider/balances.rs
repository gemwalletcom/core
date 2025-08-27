use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainBalances;
#[cfg(feature = "rpc")]
use gem_client::Client;
use primitives::AssetBalance;

use crate::provider::balances_mapper::{map_assets_balances, map_balance_coin, map_balance_staking, map_balance_tokens};
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainBalances for SuiClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(address).await?;
        let asset_balance = map_balance_coin(balance);
        Ok(asset_balance)
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let all_balances = self.get_all_balances(address).await?;
        let asset_balances = map_balance_tokens(all_balances, token_ids);
        Ok(asset_balances)
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let delegations = self.get_stake_delegations(address).await?;
        let staking_balance = map_balance_staking(delegations);
        Ok(staking_balance)
    }

    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let all_balances = self.get_all_balances(address).await?;
        let asset_balances = map_assets_balances(all_balances);
        Ok(asset_balances)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::*;
    use primitives::Chain;

    #[tokio::test]
    async fn test_sui_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;
        assert_eq!(balance.asset_id.chain, Chain::Sui);
        println!("Balance: {:?}", balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_sui_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let token_ids = vec![
            "0x5d4b302506645c37ff133b98c4b50a5ae14841659738d6d733d59d0d217a93bf::coin::COIN".to_string(), // USDC
        ];
        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids).await?;

        for balance in &balances {
            assert_eq!(balance.asset_id.chain, Chain::Sui);
            println!("Token balance: {:?}", balance);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_sui_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();

        // First check raw RPC response to see if there are any delegations
        let delegations = client.get_stake_delegations(TEST_ADDRESS.to_string()).await?;
        println!("Found {} delegations for address {}", delegations.len(), TEST_ADDRESS);

        let balance = client.get_balance_staking(TEST_ADDRESS.to_string()).await?;

        let staking_balance = balance.expect("Test address should have staking balance");
        assert_eq!(staking_balance.asset_id.chain, Chain::Sui);

        assert!(staking_balance.balance.staked > num_bigint::BigUint::from(0u32), "Staked amount should be greater than 0");

        println!("Staking balance: {} SUI", staking_balance.balance.staked);
        Ok(())
    }
}
