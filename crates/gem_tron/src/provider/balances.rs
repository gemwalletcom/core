use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::future::join_all;
use num_bigint::BigUint;
use std::error::Error;

use gem_client::Client;
use primitives::{AssetBalance, AssetId};

use crate::{provider::balances_mapper, rpc::client::TronClient};

#[async_trait]
impl<C: Client> ChainBalances for TronClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        balances_mapper::map_coin_balance(&account)
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let parameter = balances_mapper::format_address_parameter(&address)?;
        let futures: Vec<_> = token_ids
            .into_iter()
            .map(|token_id| {
                let parameter = parameter.clone();
                async move {
                    let balance_hex = self.trigger_constant_contract(&token_id, "balanceOf(address)", &parameter).await?;
                    let asset_id = AssetId::from(self.get_chain(), Some(token_id));
                    balances_mapper::map_token_balance(&balance_hex, asset_id)
                }
            })
            .collect();
        join_all(futures).await.into_iter().collect::<Result<Vec<_>, _>>()
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(Some(AssetBalance::new_staking(
            self.get_chain().as_asset_id(),
            BigUint::from(0u32),
            BigUint::from(0u32),
            BigUint::from(0u32),
        )))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_test_client, TEST_ADDRESS};
    use primitives::Chain;

    #[tokio::test]
    async fn test_tron_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;

        assert_eq!(balance.asset_id.chain, Chain::Tron);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));

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
            assert!(balance.balance.available >= num_bigint::BigUint::from(0u32));
        }

        assert!(balances.first().unwrap().balance.available > num_bigint::BigUint::from(0u32), "USDT balance should be greater than 0");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let staking_balance = client.get_balance_staking(TEST_ADDRESS.to_string()).await?;

        if let Some(balance) = staking_balance {
            assert_eq!(balance.asset_id.chain, Chain::Tron);
            assert_eq!(balance.asset_id.token_id, None);
            assert!(balance.balance.available >= num_bigint::BigUint::from(0u32));
            assert!(balance.balance.staked >= num_bigint::BigUint::from(0u32));
        }

        Ok(())
    }
}
