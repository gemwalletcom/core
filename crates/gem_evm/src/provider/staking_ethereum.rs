use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use gem_client::Client;
use num_bigint::{BigInt, Sign};
use num_traits::Zero;
use primitives::{Chain, DelegationBase, DelegationState, DelegationValidator};
use std::{error::Error, str::FromStr};

use crate::everstake::{map_balance_to_delegation, map_withdraw_request_to_delegations, IAccounting, EVERSTAKE_ACCOUNTING_ADDRESS, EVERSTAKE_POOL_ADDRESS};
use crate::multicall3::{create_call3, decode_call3_return, IMulticall3};
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
        let account = Address::from_str(address).map_err(|e| Box::new(e) as Box<dyn Error + Sync + Send>)?;
        let staker = Address::from_str(address).map_err(|e| Box::new(e) as Box<dyn Error + Sync + Send>)?;
        let calls = vec![
            create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::autocompoundBalanceOfCall { account }),
            create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::pendingDepositedBalanceOfCall { account }),
            create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::withdrawRequestCall { staker }),
        ];

        let call_count = calls.len();
        let multicall_results = self.multicall3(calls.clone()).await?;
        if multicall_results.len() != call_count {
            return Err("Unexpected number of multicall results".into());
        }

        let autocompound_balance = Self::decode_balance_result::<IAccounting::autocompoundBalanceOfCall>(&multicall_results[0]);
        let pending_deposited_balance = Self::decode_balance_result::<IAccounting::pendingDepositedBalanceOfCall>(&multicall_results[1]);
        let withdraw_request = decode_call3_return::<IAccounting::withdrawRequestCall>(&multicall_results[2])?;

        let mut delegations = Vec::new();
        let active_balance = &autocompound_balance - &pending_deposited_balance;
        if active_balance > BigInt::zero() {
            delegations.push(map_balance_to_delegation(&active_balance, DelegationState::Active));
        }
        if pending_deposited_balance > BigInt::zero() {
            delegations.push(map_balance_to_delegation(&pending_deposited_balance, DelegationState::Activating));
        }
        let withdraw_delegations = map_withdraw_request_to_delegations(&withdraw_request);
        delegations.extend_from_slice(&withdraw_delegations);

        Ok(delegations)
    }

    fn decode_balance_result<T: SolCall>(result: &IMulticall3::Result) -> BigInt
    where
        T::Return: Into<U256>,
    {
        if result.success {
            decode_call3_return::<T>(result)
                .map(|value| {
                    let value: U256 = value.into();
                    let bytes = value.to_be_bytes::<32>();
                    BigInt::from_bytes_be(Sign::Plus, &bytes)
                })
                .unwrap_or(BigInt::zero())
        } else {
            BigInt::zero()
        }
    }
}

#[cfg(all(test, feature = "rpc"))]
mod tests {
    use crate::provider::testkit::create_ethereum_test_client;
    use chain_traits::ChainStaking;

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
}
