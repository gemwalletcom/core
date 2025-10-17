use std::error::Error;

use num_bigint::BigInt;

use crate::models::ChainParameter;
use crate::models::TronAccountUsage;
use crate::rpc::constants::{
    DEFAULT_BANDWIDTH_BYTES, GET_CREATE_ACCOUNT_FEE, GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT, GET_ENERGY_FEE, GET_TRANSACTION_FEE,
};
use primitives::StakeType;

const FEE_LIMIT_PERCENT_INCREASE: u32 = 20;

pub fn calculate_transfer_fee_rate(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    is_new_account: bool,
) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let bandwidth_price = get_chain_parameter_value(chain_parameters, GET_TRANSACTION_FEE)?;

    if is_new_account {
        let activation_fee = get_chain_parameter_value(chain_parameters, GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT)?;
        let bandwidth_fee = get_chain_parameter_value(chain_parameters, GET_CREATE_ACCOUNT_FEE)?;

        let available_bandwidth = get_available_bandwidth(account_usage);
        let total_fee = if available_bandwidth >= DEFAULT_BANDWIDTH_BYTES {
            BigInt::from(activation_fee)
        } else {
            BigInt::from(activation_fee) + BigInt::from(bandwidth_fee)
        };

        Ok(total_fee)
    } else {
        Ok(calculate_fee_by_bandwidth(account_usage, DEFAULT_BANDWIDTH_BYTES, bandwidth_price))
    }
}

pub fn calculate_transfer_token_fee_rate(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    estimated_energy: BigInt,
) -> Result<(BigInt, BigInt, BigInt), Box<dyn Error + Send + Sync>> {
    let estimated_energy = estimated_energy * BigInt::from(100 + FEE_LIMIT_PERCENT_INCREASE) / BigInt::from(100);

    let energy_price = BigInt::from(get_chain_parameter_value(chain_parameters, GET_ENERGY_FEE)?);
    let bandwidth_price = get_chain_parameter_value(chain_parameters, GET_TRANSACTION_FEE)?;

    let energy = calculate_missing_energy(account_usage, &estimated_energy);
    let energy_fee = &energy * &energy_price;
    let bandwidth_fee = calculate_fee_by_bandwidth(account_usage, DEFAULT_BANDWIDTH_BYTES, bandwidth_price);

    let total_fee = energy_fee + bandwidth_fee;

    Ok((total_fee, energy, energy_price))
}

pub fn calculate_stake_fee_rate(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    _stake_type: &StakeType,
) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let bandwidth_price = get_chain_parameter_value(chain_parameters, GET_TRANSACTION_FEE)?;
    Ok(calculate_fee_by_bandwidth(account_usage, DEFAULT_BANDWIDTH_BYTES, bandwidth_price))
}

fn get_available_bandwidth(account_usage: &TronAccountUsage) -> u64 {
    let free_bandwidth = account_usage.free_net_limit.saturating_sub(account_usage.free_net_used);
    let staked_bandwidth = account_usage.net_limit.saturating_sub(account_usage.net_used);
    free_bandwidth.saturating_add(staked_bandwidth)
}

fn calculate_fee_by_bandwidth(account_usage: &TronAccountUsage, required_bandwidth: u64, bandwidth_price: i64) -> BigInt {
    let bandwidth = calculate_missing_bandwidth(account_usage, required_bandwidth);
    BigInt::from(bandwidth) * BigInt::from(bandwidth_price)
}

fn calculate_missing_bandwidth(account_usage: &TronAccountUsage, required_bandwidth: u64) -> u64 {
    let available_bandwidth = get_available_bandwidth(account_usage);
    required_bandwidth.saturating_sub(available_bandwidth)
}

fn calculate_missing_energy(account_usage: &TronAccountUsage, estimated_energy: &BigInt) -> BigInt {
    let available_energy = BigInt::from(account_usage.energy_limit.saturating_sub(account_usage.energy_used));
    std::cmp::max(BigInt::from(0), estimated_energy - available_energy)
}

fn get_chain_parameter_value(parameters: &[ChainParameter], key: &str) -> Result<i64, Box<dyn Error + Send + Sync>> {
    parameters
        .iter()
        .find(|param| param.key == key)
        .and_then(|param| param.value)
        .ok_or_else(|| format!("Missing chain parameter: {}", key).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;
    use primitives::delegation::DelegationValidator;

    fn chain_parameter(key: &str, value: i64) -> ChainParameter {
        ChainParameter {
            key: key.to_string(),
            value: Some(value),
        }
    }

    fn usage(free_limit: u64, staked_limit: u64) -> TronAccountUsage {
        TronAccountUsage {
            free_net_used: 0,
            free_net_limit: free_limit,
            net_used: 0,
            net_limit: staked_limit,
            energy_used: 0,
            energy_limit: 0,
        }
    }

    fn tron_stake_type() -> StakeType {
        StakeType::Stake(DelegationValidator {
            chain: Chain::Tron,
            id: "validator".to_string(),
            name: "validator".to_string(),
            is_active: true,
            commission: 0.0,
            apr: 0.0,
        })
    }

    #[test]
    fn test_calculate_transfer_fee_rate() {
        let transaction_fee = 1_000;
        let bandwidth_params = vec![chain_parameter(GET_TRANSACTION_FEE, transaction_fee)];

        let existing_account = usage(DEFAULT_BANDWIDTH_BYTES, 0);
        let fee = calculate_transfer_fee_rate(&bandwidth_params, &existing_account, false).unwrap();
        assert_eq!(fee, BigInt::from(0));

        let limited_account = usage(100, 0);
        let fee = calculate_transfer_fee_rate(&bandwidth_params, &limited_account, false).unwrap();
        let expected_burn = BigInt::from((DEFAULT_BANDWIDTH_BYTES - 100) as i64) * BigInt::from(transaction_fee);
        assert_eq!(fee, expected_burn);

        let activation_fee = 1_000_000;
        let bandwidth_fee = 100_000;
        let new_account_params = vec![
            chain_parameter(GET_CREATE_ACCOUNT_FEE, bandwidth_fee),
            chain_parameter(GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT, activation_fee),
            chain_parameter(GET_TRANSACTION_FEE, transaction_fee),
        ];

        let new_account_without_bandwidth = usage(0, 0);
        let fee = calculate_transfer_fee_rate(&new_account_params, &new_account_without_bandwidth, true).unwrap();
        assert_eq!(fee, BigInt::from(activation_fee + bandwidth_fee));

        let new_account_with_bandwidth = usage(200, 200);
        let fee = calculate_transfer_fee_rate(&new_account_params, &new_account_with_bandwidth, true).unwrap();
        assert_eq!(fee, BigInt::from(activation_fee));
    }

    #[test]
    fn test_calculate_transfer_token_fee_rate() {
        let params = vec![chain_parameter(GET_ENERGY_FEE, 100), chain_parameter(GET_TRANSACTION_FEE, 1_000)];
        let account_usage = usage(0, 0);
        let estimated_energy = BigInt::from(500);

        let estimated_with_buffer = &estimated_energy * BigInt::from(100 + FEE_LIMIT_PERCENT_INCREASE) / BigInt::from(100);
        let expected_bandwidth_fee = BigInt::from(DEFAULT_BANDWIDTH_BYTES) * BigInt::from(1_000);
        let expected_energy_fee = &estimated_with_buffer * BigInt::from(100);
        let expected_total = expected_bandwidth_fee + expected_energy_fee;

        let (total, missing_energy, energy_price) = calculate_transfer_token_fee_rate(&params, &account_usage, estimated_energy).unwrap();

        assert_eq!(missing_energy, estimated_with_buffer);
        assert_eq!(energy_price, BigInt::from(100));
        assert_eq!(total, expected_total);
    }

    #[test]
    fn test_get_available_bandwidth() {
        let account_usage = TronAccountUsage {
            free_net_used: 100,
            free_net_limit: 1_000,
            net_used: 50,
            net_limit: 600,
            energy_used: 0,
            energy_limit: 0,
        };

        assert_eq!(get_available_bandwidth(&account_usage), 1_450);
    }

    #[test]
    fn test_calculate_fee_by_bandwidth() {
        let enough_bandwidth = usage(500, 0);
        assert_eq!(calculate_fee_by_bandwidth(&enough_bandwidth, 400, 1_000), BigInt::from(0));

        let limited_bandwidth = usage(100, 0);
        let expected = BigInt::from((400 - 100) as i64) * BigInt::from(1_000);
        assert_eq!(calculate_fee_by_bandwidth(&limited_bandwidth, 400, 1_000), expected);
    }

    #[test]
    fn test_get_chain_parameter_value() {
        let parameters = vec![
            ChainParameter {
                key: GET_TRANSACTION_FEE.to_string(),
                value: Some(1_000),
            },
            ChainParameter {
                key: GET_ENERGY_FEE.to_string(),
                value: Some(300),
            },
        ];

        assert_eq!(get_chain_parameter_value(&parameters, GET_TRANSACTION_FEE).unwrap(), 1_000);
        assert_eq!(get_chain_parameter_value(&parameters, GET_ENERGY_FEE).unwrap(), 300);

        let err = get_chain_parameter_value(&parameters, "missing").unwrap_err();
        assert!(err.to_string().contains("Missing chain parameter"));
    }

    #[test]
    fn test_calculate_stake_fee_rate() {
        let params = vec![chain_parameter(GET_TRANSACTION_FEE, 1_000)];
        let shortfall_usage = usage(100, 0);
        let stake_type = tron_stake_type();

        let expected = BigInt::from(DEFAULT_BANDWIDTH_BYTES - 100) * BigInt::from(1_000);
        assert_eq!(calculate_stake_fee_rate(&params, &shortfall_usage, &stake_type).unwrap(), expected);

        let ample_usage = usage(DEFAULT_BANDWIDTH_BYTES, 0);
        assert_eq!(calculate_stake_fee_rate(&params, &ample_usage, &stake_type).unwrap(), BigInt::from(0));
    }

    #[test]
    fn test_calculate_chargeable_energy_respects_available_balance() {
        let mut account_usage = usage(0, 0);
        account_usage.energy_limit = 1_000;
        account_usage.energy_used = 600;

        let energy_used = BigInt::from(800);
        assert_eq!(calculate_missing_energy(&account_usage, &energy_used), BigInt::from(400));
    }

    #[test]
    fn test_calculate_transfer_token_fee_rate_with_buffer() {
        let params = vec![chain_parameter(GET_ENERGY_FEE, 420), chain_parameter(GET_TRANSACTION_FEE, 1_000)];

        let mut account_usage = usage(DEFAULT_BANDWIDTH_BYTES, 0);
        account_usage.energy_limit = 40_000;
        account_usage.energy_used = 0;

        let estimated_energy = BigInt::from(40_000);

        let (total_fee, missing_energy, energy_price) = calculate_transfer_token_fee_rate(&params, &account_usage, estimated_energy.clone()).unwrap();

        let estimated_with_buffer = estimated_energy * BigInt::from(100 + FEE_LIMIT_PERCENT_INCREASE) / BigInt::from(100);
        let expected_missing_energy = &estimated_with_buffer - BigInt::from(40_000);
        let expected_energy_fee = &expected_missing_energy * &energy_price;
        let expected_total_fee = expected_energy_fee;

        assert_eq!(missing_energy, BigInt::from(8_000));
        assert_eq!(total_fee, BigInt::from(3_360_000));
        assert_eq!(missing_energy, expected_missing_energy);
        assert_eq!(total_fee, expected_total_fee);
    }
}
