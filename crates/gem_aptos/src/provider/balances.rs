use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::future::try_join_all;
use std::{error::Error, sync::Arc};

use gem_client::Client;
use primitives::AssetBalance;

use super::balances_mapper::{map_balance_tokens, map_native_balance};
use crate::rpc::client::AptosClient;
use crate::APTOS_NATIVE_COIN;

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

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{create_aptos_test_client, TEST_ADDRESS};
    use chain_traits::ChainBalances;
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
}
