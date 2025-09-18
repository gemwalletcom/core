use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainStaking;
use primitives::{DelegationBase, DelegationValidator, EVMChain};

use crate::rpc::client::EthereumClient;
use gem_client::Client;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainStaking for EthereumClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        match self.chain {
            EVMChain::SmartChain => self.get_smartchain_staking_apy().await,
            _ => Ok(None),
        }
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        match self.chain {
            EVMChain::SmartChain => self.get_smartchain_validators(apy.unwrap_or(0.0)).await,
            _ => Ok(vec![]),
        }
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        match self.chain {
            EVMChain::SmartChain => self.get_smartchain_delegations(&address).await,
            _ => Ok(vec![]),
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{create_smartchain_test_client, TEST_SMARTCHAIN_STAKING_ADDRESS};
    use chain_traits::ChainStaking;
    use primitives::Chain;

    #[tokio::test]
    async fn test_smartchain_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let validators = client.get_staking_validators(Some(0.0)).await?;

        println!("SmartChain Validators count: {}", validators.len());
        assert!(validators.len() > 24);

        if let Some(validator) = validators.first() {
            assert_eq!(validator.chain, Chain::SmartChain);
            assert!(!validator.id.is_empty());
            assert!(!validator.name.is_empty());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let address = TEST_SMARTCHAIN_STAKING_ADDRESS.to_string();
        let delegations = client.get_staking_delegations(address).await?;

        println!("SmartChain Delegations: {:?}", delegations);

        assert!(!delegations.is_empty());

        for delegation in &delegations {
            println!(
                "Delegation - Validator: {}, Balance: {}, State: {:?}",
                delegation.validator_id, delegation.balance, delegation.state
            );
            assert_eq!(delegation.asset_id.chain, Chain::SmartChain);
        }

        Ok(())
    }
}
