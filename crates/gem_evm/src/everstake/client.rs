pub const EVERSTAKE_API_BASE_URL: &str = "https://eth-api-b2c.everstake.one";
pub const EVERSTAKE_STATS_PATH: &str = "/api/v1/stats";
pub const EVERSTAKE_VALIDATORS_QUEUE_PATH: &str = "/api/v1/validators/queue";

use super::{EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting, models::AccountState};

use alloy_primitives::Address;
use gem_client::Client;
use num_bigint::BigUint;
use num_traits::Zero;
use std::{error::Error, str::FromStr};

#[cfg(feature = "rpc")]
use crate::rpc::client::EthereumClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub async fn get_everstake_validator_queue() -> Result<super::models::QueueStatsResponse, Box<dyn Error + Send + Sync>> {
    let client = ReqwestClient::new(EVERSTAKE_API_BASE_URL.to_string(), reqwest::Client::new());
    let response = client.get(EVERSTAKE_VALIDATORS_QUEUE_PATH).await?;
    Ok(response)
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub async fn get_everstake_staking_apy() -> Result<Option<f64>, Box<dyn Error + Send + Sync>> {
    let client = ReqwestClient::new(EVERSTAKE_API_BASE_URL.to_string(), reqwest::Client::new());
    let response: super::models::StatsResponse = client.get(EVERSTAKE_STATS_PATH).await?;

    Ok(Some(response.apr * 100.0))
}

pub async fn get_everstake_account_state<C: Client + Clone>(client: &EthereumClient<C>, address: &str) -> Result<AccountState, Box<dyn Error + Sync + Send>> {
    let account = Address::from_str(address).map_err(|e| Box::new(e) as Box<dyn Error + Sync + Send>)?;
    let staker = account;
    let accounting: Address = EVERSTAKE_ACCOUNTING_ADDRESS.parse().unwrap();

    let mut batch = client.multicall();
    let h_deposited = batch.add(accounting, IAccounting::depositedBalanceOfCall { account });
    let h_pending = batch.add(accounting, IAccounting::pendingBalanceOfCall { account });
    let h_pending_deposited = batch.add(accounting, IAccounting::pendingDepositedBalanceOfCall { account });
    let h_withdraw = batch.add(accounting, IAccounting::withdrawRequestCall { staker });
    let h_restaked = batch.add(accounting, IAccounting::restakedRewardOfCall { account });

    let results = batch.execute().await.map_err(|e| e.to_string())?;

    let deposited_balance = results
        .decode::<IAccounting::depositedBalanceOfCall>(&h_deposited)
        .map(u256_to_biguint)
        .unwrap_or_else(|_| BigUint::zero());
    let pending_balance = results
        .decode::<IAccounting::pendingBalanceOfCall>(&h_pending)
        .map(u256_to_biguint)
        .unwrap_or_else(|_| BigUint::zero());
    let pending_deposited_balance = results
        .decode::<IAccounting::pendingDepositedBalanceOfCall>(&h_pending_deposited)
        .map(u256_to_biguint)
        .unwrap_or_else(|_| BigUint::zero());
    let withdraw_request = results.decode::<IAccounting::withdrawRequestCall>(&h_withdraw)?;
    let restaked_reward = results
        .decode::<IAccounting::restakedRewardOfCall>(&h_restaked)
        .map(u256_to_biguint)
        .unwrap_or_else(|_| BigUint::zero());

    Ok(AccountState {
        deposited_balance,
        pending_balance,
        pending_deposited_balance,
        withdraw_request,
        restaked_reward,
    })
}

fn u256_to_biguint(value: alloy_primitives::U256) -> BigUint {
    BigUint::from_bytes_be(&value.to_be_bytes::<32>())
}

#[cfg(all(test, feature = "rpc", feature = "reqwest", feature = "chain_integration_tests"))]
mod tests {
    use crate::everstake::client::get_everstake_validator_queue;

    #[tokio::test]
    async fn test_validator_queue() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let state = get_everstake_validator_queue().await?;

        assert!(state.validator_activation_time > 0);
        assert!(state.validator_exit_time > 0);
        assert!(state.validator_withdraw_time > 0);

        let activation_days = (state.validator_activation_time + state.validator_adding_delay) as f64 / (24 * 60 * 60) as f64;
        let withdraw_days = (state.validator_withdraw_time + state.validator_exit_time) as f64 / (24 * 60 * 60) as f64;

        println!("Ethereum activation time: {activation_days:.2} days");
        println!("Ethereum withdraw time: {withdraw_days:.2} days");

        Ok(())
    }
}
