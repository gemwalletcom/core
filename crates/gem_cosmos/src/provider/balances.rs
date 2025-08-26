use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::try_join;
use std::error::Error;

use gem_client::Client;
use primitives::{AssetBalance, AssetId};

use crate::{provider::balances_mapper, rpc::client::CosmosClient};

#[async_trait]
impl<C: Client> ChainBalances for CosmosClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balances = self.get_balances(&address).await?;
        let chain = self.get_chain().as_chain();
        let denom = chain.as_denom().ok_or("Chain does not have a denom")?;

        let balance = balances.balances.iter().find(|balance| balance.denom == denom).ok_or("Balance not found")?;

        Ok(AssetBalance::new(chain.as_asset_id(), balance.amount.to_string()))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balances = self.get_balances(&address).await?;
        let token_balances = token_ids
            .iter()
            .filter_map(|token_id| {
                balances.balances.iter().find(|balance| balance.denom == *token_id).and_then(|balance| {
                    let amount = balance.amount.parse::<u128>().ok()?;
                    let asset_id = AssetId {
                        chain: self.get_chain().as_chain(),
                        token_id: Some(token_id.clone()),
                    };
                    Some(AssetBalance::new(asset_id, amount.to_string()))
                })
            })
            .collect();

        Ok(token_balances)
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let cosmos_chain = self.get_chain();
        let chain = cosmos_chain.as_chain();
        if !chain.is_stake_supported() {
            return Ok(None);
        }
        let denom = chain.as_denom().ok_or("Chain does not have a denom")?;

        let (delegations, unbonding, rewards) = try_join!(
            self.get_delegations(&address),
            self.get_unbonding_delegations(&address),
            self.get_delegation_rewards(&address)
        )?;

        Ok(Some(balances_mapper::map_balance_staking(delegations, unbonding, rewards, chain, denom)))
    }
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use crate::provider::testkit::create_cosmos_test_client;
    use chain_traits::ChainBalances;

    const TEST_ADDRESS: &str = "cosmos1cvh8mpz04az0x7vht6h6ekksg8wd650r39ltwj";

    #[tokio::test]
    async fn test_cosmos_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_cosmos_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;
        let available = balance.balance.available.parse::<u64>().unwrap();
        assert!(available > 0);
        println!("Balance: {:?} {}", available, balance.asset_id);
        Ok(())
    }
}
