use async_trait::async_trait;
use chain_traits::ChainStaking;
use number_formatter::BigNumberFormatter;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationState, DelegationValidator};

use crate::{rpc::client::HyperCoreClient, typeshare::balance::HypercoreValidator};

#[async_trait]
impl<C: Client> ChainStaking for HyperCoreClient<C> {
    async fn get_staking_validators(&self) -> Result<Vec<DelegationValidator>, Box<dyn Error + Send + Sync>> {
        let validators = self.get_staking_validators().await?;
        let apy = HypercoreValidator::max_apr(validators.clone());
        Ok(validators
            .into_iter()
            .map(|x| DelegationValidator {
                chain: self.chain,
                id: x.validator_address(),
                name: x.name,
                is_active: x.is_active,
                commision: x.commission.parse::<f64>().unwrap_or(0.0),
                apr: apy,
            })
            .collect())
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let delegations = self
            .get_staking_delegations(&address)
            .await?
            .into_iter()
            .map(|x| DelegationBase {
                asset_id: self.chain.as_asset_id(),
                state: DelegationState::Active,
                balance: BigNumberFormatter::value_from_amount(&x.amount, 18).unwrap_or("0".to_string()),
                shares: "0".to_string(),
                rewards: "0".to_string(),
                completion_date: None,
                delegation_id: x.validator_address(),
                validator_id: x.validator_address(),
            })
            .collect();
        Ok(delegations)
    }
}
