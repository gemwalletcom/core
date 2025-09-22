use gem_client::Client;
use num_bigint::{BigInt, BigUint};
use num_traits::Zero;
use primitives::{AssetBalance, AssetId, Balance, Chain, DelegationBase, DelegationState, DelegationValidator};
use std::error::Error;

use crate::everstake::{fetch_everstake_account_state, map_balance_to_delegation, map_withdraw_request_to_delegations, EVERSTAKE_POOL_ADDRESS};
use crate::rpc::client::EthereumClient;

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumClient<C> {
    pub async fn get_ethereum_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(Some(3.2))
    }

    pub async fn get_ethereum_validators(&self, apy: f64) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        Ok(vec![DelegationValidator {
            id: EVERSTAKE_POOL_ADDRESS.to_string(),
            chain: Chain::Ethereum,
            name: "Everstake".to_string(),
            is_active: true,
            commision: 0.1,
            apr: apy,
        }])
    }

    pub async fn get_ethereum_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let state = fetch_everstake_account_state(self, address).await?;

        let mut delegations = Vec::new();

        let active_balance = &state.autocompound_balance - &state.pending_deposited_balance;
        if active_balance > BigInt::zero() {
            delegations.push(map_balance_to_delegation(&active_balance, DelegationState::Active));
        }

        if state.pending_deposited_balance > BigInt::zero() {
            delegations.push(map_balance_to_delegation(&state.pending_deposited_balance, DelegationState::Activating));
        }

        let mut withdraw_delegations = map_withdraw_request_to_delegations(&state.withdraw_request);
        delegations.append(&mut withdraw_delegations);

        Ok(delegations)
    }

    pub async fn get_ethereum_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let delegations = self.get_ethereum_delegations(address).await?;

        let mut staked = BigUint::zero();
        let mut pending_total = BigUint::zero();
        let mut withdrawable = BigUint::zero();

        for delegation in delegations {
            let balance = delegation.balance.to_biguint().unwrap_or_else(BigUint::zero);
            match delegation.state {
                DelegationState::Active => staked += balance,
                DelegationState::Activating | DelegationState::Deactivating | DelegationState::Undelegating => pending_total += balance,
                DelegationState::AwaitingWithdrawal => withdrawable += balance,
                _ => {}
            }
        }

        if staked.is_zero() && pending_total.is_zero() && withdrawable.is_zero() {
            return Ok(None);
        }

        let balance = Balance {
            available: BigUint::zero(),
            frozen: BigUint::zero(),
            locked: BigUint::zero(),
            staked,
            pending: pending_total,
            rewards: BigUint::zero(),
            reserved: BigUint::zero(),
            withdrawable,
            metadata: None,
        };

        Ok(Some(AssetBalance::new_balance(AssetId::from_chain(Chain::Ethereum), balance)))
    }
}

#[cfg(all(test, feature = "rpc"))]
mod tests {
    use crate::provider::testkit::create_ethereum_test_client;
    use chain_traits::{ChainBalances, ChainStaking};

    #[tokio::test]
    #[ignore = "Skipped by default"]
    async fn test_ethereum_get_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let address = "0x4d292dce4f83758457c09905be9ad83cd4a05447".to_string();
        let delegations = client.get_staking_delegations(address.clone()).await?;

        println!("Delegations for address: {}", address);
        for delegation in &delegations {
            println!(
                "Delegation - Validator: {}, Balance: {}, State: {:?}",
                delegation.validator_id, delegation.balance, delegation.state
            );
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Skipped by default"]
    async fn test_ethereum_get_staking_balance() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let address = "0x4d292dce4f83758457c09905be9ad83cd4a05447".to_string();
        let balance = client.get_balance_staking(address).await?;

        println!("Ethereum staking balance: {:?}", balance);

        Ok(())
    }
}
