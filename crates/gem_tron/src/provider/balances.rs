use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::future::join_all;
use std::error::Error;

use gem_client::Client;
use primitives::{AssetBalance, AssetId};

use crate::{
    provider::balances_mapper::{format_address_parameter, map_coin_balance, map_staking_balance, map_token_balance},
    rpc::{client::TronClient, trongrid::mapper::TronGridMapper},
};

#[async_trait]
impl<C: Client> ChainBalances for TronClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        map_coin_balance(&account)
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let parameter = format_address_parameter(&address)?;
        let futures: Vec<_> = token_ids
            .into_iter()
            .map(|token_id| {
                let parameter = parameter.clone();
                async move {
                    let balance_hex = self.trigger_constant_contract(&token_id, "balanceOf(address)", &parameter).await?;
                    let asset_id = AssetId::from(self.get_chain(), Some(token_id));
                    map_token_balance(&balance_hex, asset_id)
                }
            })
            .collect();
        join_all(futures).await.into_iter().collect::<Result<Vec<_>, _>>()
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let (account, reward, usage) = futures::try_join!(self.get_account(&address), self.get_reward(&address), self.get_account_usage(&address))?;
        Ok(Some(map_staking_balance(&account, &reward, &usage)?))
    }

    async fn get_balance_assets(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let account = self.trongrid_client.get_accounts_by_address(&address).await?;
        Ok(TronGridMapper::map_asset_balances(account.data.first().unwrap().clone()))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_test_client};
    use num_bigint::BigUint;
    use primitives::Chain;

    #[tokio::test]
    async fn test_tron_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;

        assert_eq!(balance.asset_id.chain, Chain::Tron);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.available > BigUint::from(0u32));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let token_ids = vec![
            "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(), // USDT
        ];

        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids.clone()).await?;

        assert_eq!(balances.len(), token_ids.len());
        for (i, balance) in balances.iter().enumerate() {
            assert_eq!(balance.asset_id.chain, Chain::Tron);
            assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
            assert!(balance.balance.available > BigUint::from(0u32));
        }

        assert!(
            balances.first().unwrap().balance.available > BigUint::from(0u32),
            "USDT balance should be greater than 0"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let balance = client.get_balance_staking(TEST_ADDRESS.to_string()).await?;

        println!("balance: {:#?}", balance);

        let balance = balance.ok_or("Staking balance not found")?;

        assert_eq!(balance.asset_id.chain, Chain::Tron);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.staked > BigUint::from(0u32));

        let metadata = balance.balance.metadata.as_ref().ok_or("Metadata not found")?;

        println!("metadata: {:#?}", metadata);

        assert!(metadata.bandwidth_available > 0);
        assert!(metadata.bandwidth_total >= 600);

        //assert!(metadata.energy_available);
        //assert!(metadata.energy_total > 0);

        assert!(metadata.bandwidth_available <= metadata.bandwidth_total);
        assert!(metadata.energy_available <= metadata.energy_total);

        Ok(())
    }

    #[tokio::test]
    async fn test_tron_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let address = TEST_ADDRESS.to_string();
        let assets = client.get_balance_assets(address).await?;

        assert!(!assets.is_empty(), "TRON test address should have TRC20 tokens");

        println!("TRON assets count: {}", assets.len());

        for asset in &assets {
            println!("Asset: {:?}", asset);
            assert_eq!(asset.asset_id.chain, Chain::Tron);
            assert!(asset.balance.available > BigUint::from(0u32));
            assert!(asset.asset_id.token_id.is_some());
        }
        Ok(())
    }
}
