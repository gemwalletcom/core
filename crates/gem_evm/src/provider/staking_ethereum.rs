use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use gem_client::Client;
use num_bigint::{BigInt, Sign};
use num_traits::Zero;
use primitives::{Chain, DelegationBase, DelegationState, DelegationValidator};
use std::{error::Error, str::FromStr};

use crate::everstake::{
    map_balance_to_delegation, map_withdraw_request_to_delegation, EverstakeAccounting, EverstakePoolQuery, EVERSTAKE_ACCOUNTING_ADDRESS,
    EVERSTAKE_POOL_ADDRESS,
};
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
            create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, EverstakeAccounting::autocompoundBalanceOfCall { account }),
            create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, EverstakeAccounting::pendingDepositedBalanceOfCall { account }),
            create_call3(EVERSTAKE_POOL_ADDRESS, EverstakePoolQuery::withdrawRequestCall { staker }),
        ];
        let call_count = calls.len();

        let multicall_results = self.multicall3(calls.clone()).await?;
        let mut delegations = Vec::new();
        if multicall_results.len() != call_count {
            return Err("Unexpected number of multicall results".into());
        }

        let autocompound_balance = Self::decode_balance_result::<EverstakeAccounting::autocompoundBalanceOfCall>(&multicall_results[0]);
        let pending_deposited_balance = Self::decode_balance_result::<EverstakeAccounting::pendingDepositedBalanceOfCall>(&multicall_results[1]);
        let pending_balance = Self::decode_balance_result::<EverstakeAccounting::pendingBalanceOfCall>(&multicall_results[2]);

        let active_balance = autocompound_balance - pending_deposited_balance;
        if active_balance > BigInt::zero() {
            delegations.push(map_balance_to_delegation(&active_balance, &BigInt::zero(), DelegationState::Active));
        }
        if pending_balance > BigInt::zero() {
            delegations.push(map_balance_to_delegation(&pending_balance, &BigInt::zero(), DelegationState::Pending));
        }

        if multicall_results[call_count - 1].success {
            if let Ok(withdraw_request) = decode_call3_return::<EverstakePoolQuery::withdrawRequestCall>(&multicall_results[3]) {
                let amount_bytes = withdraw_request.amount.to_be_bytes::<32>();
                let balance = BigInt::from_bytes_be(Sign::Plus, &amount_bytes);
                if let Some(delegation) = map_withdraw_request_to_delegation(&withdraw_request, &balance) {
                    delegations.push(delegation);
                }
            }
        }

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
