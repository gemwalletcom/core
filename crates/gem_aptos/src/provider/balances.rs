use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::future::try_join_all;
use std::{error::Error, sync::Arc};

use gem_client::Client;
use primitives::AssetBalance;

use super::balances_mapper::{map_balance_staking, map_balance_tokens, map_native_balance};
use crate::{APTOS_NATIVE_COIN, KNOWN_VALIDATOR_POOL, rpc::client::AptosClient};

#[async_trait]
impl<C: Client> ChainBalances for AptosClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_account_balance(&address, APTOS_NATIVE_COIN).await?;
        Ok(map_native_balance(&num_bigint::BigUint::from(balance), self.get_chain()))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let address_arc = Arc::new(address);
        let futures = token_ids.into_iter().map(|token_id| {
            let address = address_arc.clone();
            let client = self;
            async move {
                let balance = client.get_account_balance(&address, &token_id).await?;
                Ok::<(String, u64), Box<dyn Error + Send + Sync>>((token_id, balance))
            }
        });

        let results = try_join_all(futures).await?;

        Ok(map_balance_tokens(results, self.get_chain()))
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let stake = self.get_delegation_pool_stake(KNOWN_VALIDATOR_POOL, &address).await?;
        Ok(Some(map_balance_staking(stake, self.get_chain())))
    }

    async fn get_balance_assets(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, TEST_ADDRESS_STAKING, create_aptos_test_client};
    use chain_traits::ChainBalances;
    use num_bigint::BigUint;
    use primitives::Chain;

    #[tokio::test]
    async fn test_aptos_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;
        assert_eq!(balance.asset_id.chain, Chain::Aptos);
        println!("Balance: {:?}", balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let token_ids = vec![
            "0x159df6b7689437016108a019fd5bef736bac692b6d4a1f10c941f6fbb9a74ca6::oft::CakeOFT".to_string(), // CakeOFT
        ];

        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids.clone()).await?;

        assert_eq!(balances.len(), token_ids.len());
        for (i, balance) in balances.iter().enumerate() {
            assert_eq!(balance.asset_id.chain, Chain::Aptos);
            assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
            assert!(balance.balance.available > num_bigint::BigUint::from(0u32));
            println!("Token balance: {:?}", balance);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let address = TEST_ADDRESS.to_string();
        let assets = client.get_balance_assets(address).await?;

        assert_eq!(assets.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let balance = client.get_balance_staking(TEST_ADDRESS_STAKING.to_string()).await?;

        assert!(balance.is_some());

        if let Some(balance) = balance {
            assert_eq!(balance.asset_id.chain, Chain::Aptos);
            assert_eq!(balance.asset_id.token_id, None);
            println!(
                "Staking balance: staked={}, pending={}, rewards={}",
                balance.balance.staked, balance.balance.pending, balance.balance.rewards
            );

            assert!(balance.balance.staked > BigUint::from(0u32));
        }

        Ok(())
    }
}
