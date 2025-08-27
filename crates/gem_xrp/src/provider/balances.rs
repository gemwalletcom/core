use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use crate::{
    provider::balances_mapper::{map_balance_coin, map_balance_tokens},
    rpc::client::XRPClient,
};

#[async_trait]
impl<C: Client> ChainBalances for XRPClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = self.get_account_info(&address).await?;
        let reserved_amount = self.get_chain().account_activation_fee().unwrap_or(0) as u64;

        map_balance_coin(&account, self.get_chain().as_asset_id(), reserved_amount)
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let objects = self.get_account_objects(&address).await?;
        Ok(map_balance_tokens(&objects, token_ids, self.get_chain()))
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use primitives::Chain;

    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_xrp_test_client};

    #[tokio::test]
    async fn test_xrp_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));
        println!("Balance: {:?} {}", balance.balance.available, balance.asset_id);
        Ok(())
    }

    #[tokio::test]
    async fn test_xrp_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let token_ids = vec![
            "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De".to_string(), // RLUSD
        ];
        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids).await?;

        assert_eq!(balances.len(), 1);
        for balance in &balances {
            assert_eq!(balance.asset_id.chain, Chain::Xrp);

            println!("Token balance: {:?}", balance);
            assert!(balance.balance.available > num_bigint::BigUint::from(0u32));
        }
        Ok(())
    }
}
