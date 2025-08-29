use async_trait::async_trait;
use chain_traits::ChainStaking;
use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{Asset, AssetId, Chain, DelegationBase, DelegationState, DelegationValidator};

use super::staking_mapper::map_staking_validators;
use crate::rpc::client::TronClient;
use crate::rpc::constants::{GET_WITNESS_PAY_PER_BLOCK, GET_WITNESS_127_PAY_PER_BLOCK};

const SYSTEM_VALIDATOR_ID: &str = "system";

#[async_trait]
impl<C: Client + Clone> ChainStaking for TronClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let params = self.get_chain_parameters().await?;
        let witnesses = self.get_witnesses_list().await?;

        let block_reward = params
            .iter()
            .find(|p| p.key == GET_WITNESS_PAY_PER_BLOCK)
            .and_then(|p| p.value)
            .unwrap_or(16_000_000) as f64
            / 1_000_000.0;

        let voting_reward = params
            .iter()
            .find(|p| p.key == GET_WITNESS_127_PAY_PER_BLOCK)
            .and_then(|p| p.value)
            .unwrap_or(160_000_000) as f64
            / 1_000_000.0;

        let blocks_per_year = 365.25 * 24.0 * 60.0 * 60.0 / 3.0;
        let annual_rewards = (block_reward + voting_reward) * blocks_per_year;

        let total_votes: i64 = witnesses.witnesses.iter().map(|x| x.vote_count.unwrap_or(0)).sum();
        let total_staked_trx = total_votes as f64;

        if total_staked_trx == 0.0 {
            return Ok(Some(0.0));
        }

        let apy = (annual_rewards / total_staked_trx) * 100.0;

        Ok(Some(apy))
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let witnesses = self.get_witnesses_list().await?;
        Ok(map_staking_validators(witnesses, apy))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let account_future = self.get_account(&address);
        let reward_future = self.get_reward(&address);
        let validators_future = self.get_staking_validators(Some(0.0));

        let (account, reward, validators) = futures::try_join!(account_future, reward_future, validators_future)?;

        let mut delegations = Vec::new();
        let asset_id = AssetId::from(Chain::Tron, None);

        if let Some(unfrozen_v2) = account.unfrozen_v2 {
            for unfrozen in unfrozen_v2 {
                if let (Some(expire_time), Some(amount)) = (unfrozen.unfreeze_expire_time, unfrozen.unfreeze_amount) {
                    let completion_date = DateTime::from_timestamp((expire_time / 1000) as i64, 0).unwrap_or_else(Utc::now);

                    let now = Utc::now();
                    let state = if now < completion_date {
                        DelegationState::Pending
                    } else {
                        DelegationState::AwaitingWithdrawal
                    };

                    delegations.push(DelegationBase {
                        asset_id: asset_id.clone(),
                        state,
                        balance: BigInt::from(amount),
                        shares: BigInt::from(0),
                        rewards: BigInt::from(0),
                        completion_date: Some(completion_date),
                        delegation_id: completion_date.timestamp().to_string(),
                        validator_id: SYSTEM_VALIDATOR_ID.to_string(),
                    });
                }
            }
        }

        if let Some(votes) = account.votes {
            let total_votes: u64 = votes.iter().map(|v| v.vote_count).sum();
            let reward_amount = reward.reward.unwrap_or(0);

            for vote in votes {
                if validators.iter().any(|v| v.id == vote.vote_address) {
                    let proportional_reward = if total_votes > 0 {
                        (reward_amount as f64 * vote.vote_count as f64 / total_votes as f64) as u64
                    } else {
                        0
                    };
                    let balance = vote.vote_count * 10_u64.pow(Asset::from_chain(Chain::Tron).decimals as u32);

                    delegations.push(DelegationBase {
                        asset_id: asset_id.clone(),
                        state: DelegationState::Active,
                        balance: BigInt::from(balance),
                        shares: BigInt::from(vote.vote_count),
                        rewards: BigInt::from(proportional_reward),
                        completion_date: None,
                        delegation_id: format!("vote_{}", vote.vote_address),
                        validator_id: vote.vote_address,
                    });
                }
            }
        }

        Ok(delegations)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod integration_tests {
    use super::*;
    use crate::provider::testkit::{create_test_client, TEST_ADDRESS};

    #[tokio::test]
    async fn test_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let apy = client.get_staking_apy().await?;

        println!("TRON APY: {}", apy.unwrap_or(0.0));

        assert!(apy.is_some());
        let apy_value = apy.unwrap();
        assert!(apy_value > 0.0 || apy_value < 50.0);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let apy = client.get_staking_apy().await?;
        let validators = client.get_staking_validators(apy).await?;

        println!("TRON validators count: {}", validators.len());

        assert!(!validators.is_empty());
        assert!(validators.len() > 27);
        let system_validator = validators.iter().find(|v| v.id == "system");
        assert!(system_validator.is_some());
        assert_eq!(system_validator.unwrap().name, "Unstaking");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let delegations = client.get_staking_delegations(TEST_ADDRESS.to_string()).await?;
        for delegation in &delegations {
            assert_eq!(delegation.asset_id.chain, Chain::Tron);
            assert!(delegation.balance >= BigInt::from(0));
            assert!(delegation.rewards >= BigInt::from(0));
            assert!(delegation.shares >= BigInt::from(0));
        }
        Ok(())
    }
}
