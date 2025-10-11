use async_trait::async_trait;
use chain_traits::ChainStaking;
use futures::try_join;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator};

use super::staking_mapper;
use crate::{
    KNOWN_VALIDATOR_POOL,
    provider::staking_mapper::{calculate_apy, map_validators},
    rpc::client::AptosClient,
};

#[async_trait]
impl<C: Client> ChainStaking for AptosClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let staking_config = self.get_staking_config().await?;
        Ok(Some(calculate_apy(&staking_config)))
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let (validator_set, commission) = try_join!(
            self.get_validator_set(),
            self.get_operator_commission_percentage(KNOWN_VALIDATOR_POOL)
        )?;

        Ok(map_validators(validator_set, apy.unwrap_or(0.0), KNOWN_VALIDATOR_POOL, commission))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let (delegation, reconfig, lockup_secs) = try_join!(
            self.get_delegation_for_pool(&address, KNOWN_VALIDATOR_POOL),
            self.get_reconfiguration_state(),
            self.get_stake_lockup_secs(KNOWN_VALIDATOR_POOL)
        )?;
        Ok(staking_mapper::map_delegations(vec![delegation], &reconfig, lockup_secs))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_aptos_test_client};

    #[tokio::test]
    async fn test_aptos_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let apy = client.get_staking_apy().await?;
        assert!(apy.is_some());

        println!("Aptos APY: {:?}", apy);

        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let validators = client.get_staking_validators(Some(5.0)).await?;

        println!("{:?}", validators);

        assert!(!validators.is_empty());

        if let Some(first) = validators.first() {
            assert!(first.commission > 0.0);
        }

        println!("Found {} validators", validators.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let delegations = client.get_staking_delegations(TEST_ADDRESS.to_string()).await?;

        println!("Delegations: {:?}", delegations);

        assert!(!delegations.is_empty(), "Expected at least one delegation");

        for delegation in &delegations {
            println!(
                "State: {:?}, Balance: {}, Validator: {}",
                delegation.state, delegation.balance, delegation.validator_id
            );
            if let Some(date) = delegation.completion_date {
                println!("Completion date: {}", date);
            }
        }

        Ok(())
    }
}
