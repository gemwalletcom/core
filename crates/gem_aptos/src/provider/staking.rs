use async_trait::async_trait;
use chain_traits::ChainStaking;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator};

use super::staking_mapper;
use crate::{
    provider::staking_mapper::{calculate_apy, map_validators},
    rpc::client::AptosClient,
    KNOWN_VALIDATOR_POOL,
};

#[async_trait]
impl<C: Client> ChainStaking for AptosClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let staking_config = self.get_staking_config().await?;
        Ok(Some(calculate_apy(&staking_config)))
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let validator_set = self.get_validator_set().await?;
        let commission = self.get_operator_commission_percentage(KNOWN_VALIDATOR_POOL).await?;

        Ok(map_validators(validator_set, apy.unwrap_or(0.0), KNOWN_VALIDATOR_POOL, commission))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let delegation = self.get_delegation_for_pool(&address, KNOWN_VALIDATOR_POOL).await?;
        Ok(staking_mapper::map_delegations(vec![delegation]))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_aptos_test_client;

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
}
