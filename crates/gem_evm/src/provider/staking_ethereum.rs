use gem_client::Client;
use num_bigint::BigUint;
use num_traits::Zero;
use primitives::{AssetBalance, AssetId, Balance, Chain, DelegationBase, DelegationState, DelegationValidator};
use std::error::Error;

use crate::everstake::{EVERSTAKE_POOL_ADDRESS, get_everstake_account_state, map_balance_to_delegation, map_withdraw_request_to_delegations};
use crate::rpc::client::EthereumClient;

#[cfg(all(feature = "rpc", feature = "reqwest"))]
use crate::everstake::client::get_everstake_staking_apy;

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumClient<C> {
    pub async fn get_ethereum_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        #[cfg(feature = "reqwest")]
        {
            get_everstake_staking_apy().await
        }

        #[cfg(not(feature = "reqwest"))]
        {
            Ok(None)
        }
    }

    pub async fn get_ethereum_validators(&self, apy: f64) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        Ok(vec![DelegationValidator {
            id: EVERSTAKE_POOL_ADDRESS.to_string(),
            chain: Chain::Ethereum,
            name: "Everstake".to_string(),
            is_active: true,
            commission: 0.1,
            apr: apy,
        }])
    }

    pub async fn get_ethereum_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let state = get_everstake_account_state(self, address).await?;

        let mut delegations = Vec::new();

        let active_balance = state.deposited_balance;
        if active_balance > BigUint::zero() {
            delegations.push(map_balance_to_delegation(&active_balance, &state.restaked_reward, DelegationState::Active));
        }

        let pending_balance = state.pending_balance + state.pending_deposited_balance;
        if pending_balance > BigUint::zero() {
            delegations.push(map_balance_to_delegation(&pending_balance, &BigUint::zero(), DelegationState::Activating));
        }

        let mut withdraw_delegations = map_withdraw_request_to_delegations(&state.withdraw_request);
        delegations.append(&mut withdraw_delegations);

        Ok(delegations)
    }

    pub async fn get_ethereum_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let delegations = self.get_ethereum_delegations(address).await?;

        let mut staked = BigUint::zero();
        let mut rewards = BigUint::zero();
        let mut pending = BigUint::zero();
        for delegation in &delegations {
            match delegation.state {
                DelegationState::Active => {
                    staked += &delegation.balance;
                    rewards += &delegation.rewards;
                }
                DelegationState::Activating | DelegationState::Deactivating | DelegationState::AwaitingWithdrawal => {
                    pending += &delegation.balance;
                }
                _ => {}
            }
        }

        let balance = Balance::stake_balance(staked, pending, Some(rewards));

        Ok(Some(AssetBalance::new_balance(AssetId::from_chain(Chain::Ethereum), balance)))
    }
}

#[cfg(all(test, feature = "rpc"))]
mod tests {
    use crate::provider::testkit::{TEST_ADDRESS, create_ethereum_test_client};
    use chain_traits::{ChainBalances, ChainStaking};
    use num_bigint::BigUint;

    #[tokio::test]
    async fn test_ethereum_get_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let address = TEST_ADDRESS.to_string();
        let delegations = client.get_staking_delegations(address.clone()).await?;

        println!("Delegations for address: {}", address);
        for delegation in &delegations {
            println!(
                "Delegation - Validator: {}, Balance: {}, Rewards: {}, State: {:?}",
                delegation.validator_id, delegation.balance, delegation.rewards, delegation.state
            );
        }

        assert_eq!(delegations.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_staking_balance() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_staking(address).await?;

        println!("Ethereum staking balance: {:?}", balance);

        assert!(balance.unwrap().balance.staked > BigUint::from(0u32));

        Ok(())
    }
}
