use async_trait::async_trait;
use chain_traits::ChainStaking;
use futures::try_join;
use std::error::Error;

use gem_client::Client;
use primitives::{chain_cosmos::CosmosChain, DelegationBase, DelegationValidator};

use crate::{
    provider::staking_mapper::{calculate_network_apy_cosmos, calculate_network_apy_osmosis, map_staking_delegations, map_staking_validators},
    rpc::client::CosmosClient,
};

#[async_trait]
impl<C: Client> ChainStaking for CosmosClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain();
        match chain {
            CosmosChain::Noble | CosmosChain::Thorchain => Ok(None),
            CosmosChain::Cosmos | CosmosChain::Injective => {
                let (inflation, staking_pool) = try_join!(self.get_inflation(), self.get_staking_pool())?;
                Ok(calculate_network_apy_cosmos(inflation, staking_pool))
            }
            CosmosChain::Osmosis => {
                let (mint_params, epoch_provisions, staking_pool, supply) = try_join!(
                    self.get_osmosis_mint_params(),
                    self.get_osmosis_epoch_provisions(),
                    self.get_staking_pool(),
                    self.get_supply_by_denom("uosmo")
                )?;

                Ok(calculate_network_apy_osmosis(mint_params, epoch_provisions, staking_pool, supply))
            }
            CosmosChain::Celestia => Ok(Some(10.55)),
            CosmosChain::Sei => Ok(Some(5.62)),
        }
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain();
        match chain {
            CosmosChain::Noble | CosmosChain::Thorchain => Ok(vec![]),
            CosmosChain::Cosmos | CosmosChain::Injective | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Sei => {
                let validators = self.get_validators().await?;
                Ok(map_staking_validators(validators.validators, chain, apy))
            }
        }
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain().as_chain();
        let denom = chain.as_denom().unwrap_or_default();
        let chain = self.get_chain();
        match chain {
            CosmosChain::Noble | CosmosChain::Thorchain => Ok(vec![]),
            CosmosChain::Cosmos | CosmosChain::Injective | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Sei => {
                let (active_delegations, unbonding, rewards) = try_join!(
                    self.get_delegations(&address),
                    self.get_unbonding_delegations(&address),
                    self.get_delegation_rewards(&address)
                )?;

                let validators = self.get_validators().await?;

                Ok(map_staking_delegations(
                    active_delegations,
                    unbonding,
                    rewards,
                    validators.validators,
                    chain,
                    denom,
                ))
            }
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{create_cosmos_test_client, create_osmosis_test_client};
    use chain_traits::ChainStaking;

    #[tokio::test]
    async fn test_get_osmosis_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_osmosis_test_client();
        let apy = client.get_staking_apy().await?;

        assert!(apy.is_some());
        let apy_value = apy.unwrap();

        assert!(apy_value > 1.0 && apy_value < 2.0, "APY should be between 1% and 2%, got: {}", apy_value);
        assert_ne!(apy_value, 14.0);

        println!("Osmosis staking APY: {}%", apy_value);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cosmos_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_cosmos_test_client();
        let apy = client.get_staking_apy().await?;

        assert!(apy.is_some());
        let apy_value = apy.unwrap();

        assert!(apy_value > 5.0 && apy_value < 25.0);

        println!("Cosmos staking APY: {}%", apy_value);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cosmos_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_cosmos_test_client();
        let apy = client.get_staking_apy().await?;
        let validators = client.get_staking_validators(apy).await?;

        assert!(!validators.is_empty());
        assert!(validators.len() <= 200);

        for validator in validators.iter().take(5) {
            assert!(!validator.id.is_empty());
            assert!(!validator.name.is_empty());
            assert!(validator.commision >= 0.0 && validator.commision <= 100.0);
            if validator.is_active {
                assert!(validator.apr >= 0.0);
            }
        }

        Ok(())
    }
}
