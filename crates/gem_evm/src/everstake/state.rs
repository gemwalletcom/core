use alloy_primitives::Address;
use gem_client::Client;
use num_bigint::BigUint;
use num_traits::Zero;
use std::{error::Error, str::FromStr};

use crate::multicall3::{IMulticall3, create_call3, decode_call3_return};
use crate::rpc::client::EthereumClient;

use super::{EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting, WithdrawRequest};

pub struct EverstakeAccountState {
    pub deposited_balance: BigUint,
    pub pending_balance: BigUint,
    pub pending_deposited_balance: BigUint,
    pub withdraw_request: WithdrawRequest,
    pub restaked_reward: BigUint,
}

pub async fn get_everstake_account_state<C: Client + Clone>(
    client: &EthereumClient<C>,
    address: &str,
) -> Result<EverstakeAccountState, Box<dyn Error + Sync + Send>> {
    let account = Address::from_str(address).map_err(|e| Box::new(e) as Box<dyn Error + Sync + Send>)?;
    let staker = account;

    let calls = vec![
        create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::depositedBalanceOfCall { account }),
        create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::pendingBalanceOfCall { account }),
        create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::pendingDepositedBalanceOfCall { account }),
        create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::withdrawRequestCall { staker }),
        create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::restakedRewardOfCall { account }),
    ];

    let call_count = calls.len();
    let multicall_results = client.multicall3(calls).await?;
    if multicall_results.len() != call_count {
        return Err("Unexpected number of multicall results".into());
    }

    let deposited_balance = decode_balance_result::<IAccounting::depositedBalanceOfCall>(&multicall_results[0]);
    let pending_balance = decode_balance_result::<IAccounting::pendingBalanceOfCall>(&multicall_results[1]);
    let pending_deposited_balance = decode_balance_result::<IAccounting::pendingDepositedBalanceOfCall>(&multicall_results[2]);
    let withdraw_request = decode_call3_return::<IAccounting::withdrawRequestCall>(&multicall_results[3])?;
    let restaked_reward = decode_balance_result::<IAccounting::restakedRewardOfCall>(&multicall_results[4]);

    Ok(EverstakeAccountState {
        deposited_balance,
        pending_balance,
        pending_deposited_balance,
        withdraw_request,
        restaked_reward,
    })
}

fn decode_balance_result<T: alloy_sol_types::SolCall>(result: &IMulticall3::Result) -> BigUint
where
    T::Return: Into<alloy_primitives::U256>,
{
    if result.success {
        decode_call3_return::<T>(result)
            .map(|value| {
                let value: alloy_primitives::U256 = value.into();
                let bytes = value.to_be_bytes::<32>();
                BigUint::from_bytes_be(&bytes)
            })
            .unwrap_or(BigUint::zero())
    } else {
        BigUint::zero()
    }
}
