use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use crate::provider::balances_mapper::{map_balance_tokens, map_coin_balance};
use crate::rpc::client::TonClient;

#[async_trait]
impl<C: Client> ChainBalances for TonClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(address).await?;
        Ok(map_coin_balance(balance))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balances = self.get_jetton_wallets(address).await?;
        Ok(map_balance_tokens(balances, token_ids))
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::ChainBalances;
    use primitives::Chain;

    #[tokio::test]
    async fn test_ton_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ton_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;
        assert_eq!(balance.asset_id.chain, Chain::Ton);
        println!("Balance: {:?}", balance);
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));
        Ok(())
    }

    #[tokio::test]
    async fn test_ton_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ton_test_client();
        let token_ids = vec![
            "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs".to_string(), // USDT
        ];
        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids).await?;

        assert_eq!(balances.len(), 1);
        for balance in &balances {
            assert_eq!(balance.asset_id.chain, Chain::Ton);

            println!("Token balance: {:?}", balance);
            assert!(balance.balance.available > num_bigint::BigUint::from(0u32));
        }
        Ok(())
    }
}
