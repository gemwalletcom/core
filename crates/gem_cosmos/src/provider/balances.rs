use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;
use futures::try_join;

use gem_client::Client;
use primitives::{AssetBalance, AssetId};

use crate::{rpc::client::CosmosClient, provider::balances_mapper};

#[async_trait]
impl<C: Client> ChainBalances for CosmosClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balances = self.get_balances(&address).await?;
        let chain = self.get_chain().as_chain();
        let denom = chain.as_denom().ok_or("Chain does not have a denom")?;
        
        let balance = balances.balances
            .iter()
            .find(|balance| balance.denom == denom)
            .ok_or("Balance not found")?;

        Ok(AssetBalance::new(chain.as_asset_id(), balance.amount.to_string()))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balances = self.get_balances(&address).await?;        
        let token_balances = token_ids.iter().filter_map(|token_id| {
            balances.balances
                .iter()
                .find(|balance| balance.denom == *token_id)
                .and_then(|balance| {
                    let amount = balance.amount.parse::<u128>().ok()?;
                    let asset_id = AssetId {
                        chain: self.get_chain().as_chain(),
                        token_id: Some(token_id.clone()),
                    };
                    Some(AssetBalance::new(asset_id, amount.to_string()))
                })
        }).collect();

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

        Ok(Some(balances_mapper::map_balance_staking(
            delegations,
            unbonding,
            rewards,
            chain,
            &denom  
        )))
    }
}