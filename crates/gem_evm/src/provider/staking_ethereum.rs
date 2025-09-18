use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use gem_client::Client;
use primitives::{Chain, DelegationBase, DelegationState, DelegationValidator};
use std::{error::Error, str::FromStr};

use crate::everstake::{
    combine_active_balances, map_balance_string_to_delegation, map_withdraw_request_to_delegation, EverstakeAccounting, EverstakePoolQuery,
    EVERSTAKE_ACCOUNTING_ADDRESS, EVERSTAKE_POOL_ADDRESS,
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
            create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, EverstakeAccounting::depositedBalanceOfCall { account }),
            create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, EverstakeAccounting::pendingBalanceOfCall { account }),
            create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, EverstakeAccounting::autocompoundBalanceOfCall { account }),
            create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, EverstakeAccounting::pendingRestakedRewardOfCall { account }),
            create_call3(EVERSTAKE_POOL_ADDRESS, EverstakePoolQuery::withdrawRequestCall { staker }),
        ];

        let multicall_results = self.multicall3(calls).await?;
        let mut delegations = Vec::new();

        // Process multicall3 results
        if multicall_results.len() != 5 {
            return Err("Unexpected number of multicall results".into());
        }

        // Decode balance results
        let deposited_balance = Self::decode_balance_result::<EverstakeAccounting::depositedBalanceOfCall>(&multicall_results[0]);
        let pending_balance = Self::decode_balance_result::<EverstakeAccounting::pendingBalanceOfCall>(&multicall_results[1]);
        let autocompound_balance = Self::decode_balance_result::<EverstakeAccounting::autocompoundBalanceOfCall>(&multicall_results[2]);

        // Combine active balances (deposited + autocompound) into single active delegation
        if let Some(total_active_balance) = combine_active_balances(&deposited_balance, &autocompound_balance) {
            if let Some(delegation) = map_balance_string_to_delegation(EVERSTAKE_POOL_ADDRESS, &total_active_balance, DelegationState::Active) {
                delegations.push(delegation);
            }
        }

        // Add pending balance as separate delegation
        if let Some(delegation) = map_balance_string_to_delegation(EVERSTAKE_POOL_ADDRESS, &pending_balance, DelegationState::Pending) {
            delegations.push(delegation);
        }

        // Add withdrawal request as unstaking delegation
        if multicall_results[4].success {
            if let Ok(withdraw_request) = decode_call3_return::<EverstakePoolQuery::withdrawRequestCall>(&multicall_results[4]) {
                if let Some(delegation) = map_withdraw_request_to_delegation(EVERSTAKE_POOL_ADDRESS, &withdraw_request) {
                    delegations.push(delegation);
                }
            }
        }

        Ok(delegations)
    }

    fn decode_balance_result<T: SolCall>(result: &IMulticall3::Result) -> String
    where
        T::Return: ToString,
    {
        if result.success {
            decode_call3_return::<T>(result).map(|r| r.to_string()).unwrap_or_else(|_| "0".to_string())
        } else {
            "0".to_string()
        }
    }
}
