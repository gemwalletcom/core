use num_bigint::BigInt;
use std::error::Error;

use crate::models::TronAccountUsage;
use crate::models::ChainParameter;
use crate::rpc::constants::{BASE_FEE, GET_CREATE_ACCOUNT_FEE, GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT, GET_ENERGY_FEE};

pub fn calculate_transfer_fee_rate(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    is_new_account: bool,
) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let new_account_fee = get_chain_parameter_value(chain_parameters, GET_CREATE_ACCOUNT_FEE)?;
    let new_account_fee_in_smart_contract = get_chain_parameter_value(chain_parameters, GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT)?;

    let available_bandwidth = (account_usage.free_net_limit.unwrap_or(0)) - (account_usage.free_net_used.unwrap_or(0));
    let coin_transfer_fee = if available_bandwidth >= 300 {
        BigInt::from(0)
    } else {
        BigInt::from(BASE_FEE)
    };

    Ok(if is_new_account {
        coin_transfer_fee + BigInt::from(new_account_fee + new_account_fee_in_smart_contract)
    } else {
        coin_transfer_fee
    })
}

pub fn calculate_transfer_token_fee_rate(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    is_new_account: bool,
    gas_limit: BigInt,
) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let energy_fee = get_chain_parameter_value(chain_parameters, GET_ENERGY_FEE)?;
    let new_account_fee_in_smart_contract = get_chain_parameter_value(chain_parameters, GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT)?;

    let available_energy = BigInt::from(account_usage.energy_limit.unwrap_or(0)) - BigInt::from(account_usage.energy_used.unwrap_or(0));
    let energy_shortfall = std::cmp::max(BigInt::from(0), increase_by_percent(&gas_limit, 20) - available_energy);
    let token_transfer_fee = BigInt::from(energy_fee) * energy_shortfall;

    Ok(if is_new_account {
        token_transfer_fee + BigInt::from(new_account_fee_in_smart_contract)
    } else {
        token_transfer_fee
    })
}

fn increase_by_percent(value: &BigInt, percent: u32) -> BigInt {
    value + (value * BigInt::from(percent) / BigInt::from(100))
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

    #[test]
    fn test_calculate_native_transfer_fee_rate() {
        let parameters = vec![
            ChainParameter {
                key: GET_CREATE_ACCOUNT_FEE.to_string(),
                value: Some(100000),
            },
            ChainParameter {
                key: GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT.to_string(),
                value: Some(0),
            },
        ];
        let account_usage = TronAccountUsage {
            free_net_used: Some(0),
            free_net_limit: Some(1500),
            energy_used: Some(0),
            energy_limit: Some(0),
        };
        let result = calculate_transfer_fee_rate(&parameters, &account_usage, false);
        assert!(result.is_ok());
        let fee = result.unwrap();
        assert!(fee >= BigInt::from(0));
    }

    #[test]
    fn test_calculate_trc20_transfer_fee_rate() {
        let parameters = vec![
            ChainParameter {
                key: GET_ENERGY_FEE.to_string(),
                value: Some(420),
            },
            ChainParameter {
                key: GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT.to_string(),
                value: Some(0),
            },
        ];
        let account_usage = TronAccountUsage {
            free_net_used: Some(0),
            free_net_limit: Some(1500),
            energy_used: Some(0),
            energy_limit: Some(32000),
        };
        let gas_limit = BigInt::from(32000); // Reasonable default for TRC20 transfers
        let result = calculate_transfer_token_fee_rate(&parameters, &account_usage, false, gas_limit);
        assert!(result.is_ok());
        let fee = result.unwrap();
        assert!(fee >= BigInt::from(0));
    }

    #[test]
    fn test_get_chain_parameter_value() {
        let parameters = vec![ChainParameter {
            key: "getEnergyFee".to_string(),
            value: Some(500),
        }];
        let fee = get_chain_parameter_value(&parameters, GET_ENERGY_FEE);
        assert!(fee.is_ok());
        assert_eq!(fee.unwrap(), 500);
    }

    #[test]
    fn test_increase_by_percent() {
        let value = BigInt::from(1000);
        let increased = increase_by_percent(&value, 20);
        assert_eq!(increased, BigInt::from(1200)); // 1000 + (1000 * 20 / 100) = 1200
    }
}
