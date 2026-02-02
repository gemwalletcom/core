use std::error::Error;

use num_bigint::BigInt;

use crate::models::account::TronFrozen;
use crate::models::ChainParameter;
use crate::models::TronAccountUsage;
use crate::rpc::constants::{DEFAULT_BANDWIDTH_BYTES, GET_CREATE_ACCOUNT_FEE, GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT, GET_ENERGY_FEE, GET_TRANSACTION_FEE};
use primitives::{Resource, StakeType, TronUnfreeze};

const FEE_LIMIT_BUFFER_PERCENT: u64 = 20;

pub fn calculate_transfer_fee_rate(chain_parameters: &[ChainParameter], account_usage: &TronAccountUsage, is_new_account: bool) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let bandwidth_price = get_chain_parameter_value(chain_parameters, GET_TRANSACTION_FEE)?;

    if is_new_account {
        let activation_fee = get_chain_parameter_value(chain_parameters, GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT)?;
        let bandwidth_fee = get_chain_parameter_value(chain_parameters, GET_CREATE_ACCOUNT_FEE)?;

        let available_bandwidth = account_usage.available_bandwidth();
        let total_fee = if available_bandwidth >= DEFAULT_BANDWIDTH_BYTES {
            BigInt::from(activation_fee)
        } else {
            BigInt::from(activation_fee) + BigInt::from(bandwidth_fee)
        };

        Ok(total_fee)
    } else {
        let fee = bandwidth_fee(account_usage, DEFAULT_BANDWIDTH_BYTES, bandwidth_price as u64);
        Ok(BigInt::from(fee))
    }
}

pub fn calculate_transfer_token_fee_rate(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    estimated_energy: u64,
) -> Result<TokenTransferFee, Box<dyn Error + Send + Sync>> {
    let energy_price = get_chain_parameter_value(chain_parameters, GET_ENERGY_FEE)? as u64;
    let bandwidth_price = get_chain_parameter_value(chain_parameters, GET_TRANSACTION_FEE)? as u64;

    let fee = calculate_token_transfer_fee(account_usage, estimated_energy, energy_price, bandwidth_price);
    Ok(fee)
}

pub fn calculate_stake_fee_rate(chain_parameters: &[ChainParameter], account_usage: &TronAccountUsage, _stake_type: &StakeType) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let bandwidth_price = get_chain_parameter_value(chain_parameters, GET_TRANSACTION_FEE)? as u64;
    let fee = bandwidth_fee(account_usage, DEFAULT_BANDWIDTH_BYTES, bandwidth_price);
    Ok(BigInt::from(fee))
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTransferFee {
    pub fee: u64,
    pub fee_limit: u64,
    pub energy_price: u64,
}

pub fn calculate_token_transfer_fee(account_usage: &TronAccountUsage, estimated_energy: u64, energy_price: u64, bandwidth_price: u64) -> TokenTransferFee {
    let energy_with_buffer = apply_buffer(estimated_energy, FEE_LIMIT_BUFFER_PERCENT);
    let chargeable_energy = account_usage.missing_energy(energy_with_buffer);

    let energy_fee = chargeable_energy * energy_price;
    let bandwidth_fee = bandwidth_fee(account_usage, DEFAULT_BANDWIDTH_BYTES, bandwidth_price);

    TokenTransferFee {
        fee: energy_fee + bandwidth_fee,
        fee_limit: energy_with_buffer * energy_price,
        energy_price,
    }
}

fn bandwidth_fee(account_usage: &TronAccountUsage, required: u64, price: u64) -> u64 {
    if account_usage.available_bandwidth() >= required { 0 } else { required * price }
}

fn apply_buffer(value: u64, percent: u64) -> u64 {
    value * (100 + percent) / 100
}

fn get_chain_parameter_value(parameters: &[ChainParameter], key: &str) -> Result<i64, Box<dyn Error + Send + Sync>> {
    parameters
        .iter()
        .find(|param| param.key == key)
        .and_then(|param| param.value)
        .ok_or_else(|| format!("Missing chain parameter: {}", key).into())
}

pub fn calculate_unfreeze_amounts(frozen: Option<&Vec<TronFrozen>>, total: u64) -> Vec<TronUnfreeze> {
    frozen
        .map(|frozen| {
            let mut items: Vec<_> = frozen.iter().filter(|f| f.amount > 0).collect();
            items.sort_by_key(|f| std::cmp::Reverse(f.amount));

            items
                .into_iter()
                .scan(total, |remaining, f| {
                    (*remaining > 0).then(|| {
                        let take = (*remaining).min(f.amount);
                        *remaining -= take;
                        TronUnfreeze {
                            resource: match f.frozen_type.as_deref() {
                                Some("ENERGY") => Resource::Energy,
                                _ => Resource::Bandwidth,
                            },
                            amount: take,
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

impl TronAccountUsage {
    pub fn available_bandwidth(&self) -> u64 {
        let free = self.free_net_limit.saturating_sub(self.free_net_used);
        let staked = self.net_limit.saturating_sub(self.net_used);
        free.saturating_add(staked)
    }

    pub fn missing_bandwidth(&self, required: u64) -> u64 {
        required.saturating_sub(self.available_bandwidth())
    }

    pub fn available_energy(&self) -> u64 {
        self.energy_limit.saturating_sub(self.energy_used)
    }

    pub fn missing_energy(&self, required: u64) -> u64 {
        required.saturating_sub(self.available_energy())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::TronFrozen;
    use primitives::delegation::DelegationValidator;
    use primitives::Chain;

    fn chain_parameter(key: &str, value: i64) -> ChainParameter {
        ChainParameter {
            key: key.to_string(),
            value: Some(value),
        }
    }

    fn account_usage(free_bandwidth: u64, staked_bandwidth: u64, available_energy: u64) -> TronAccountUsage {
        TronAccountUsage {
            free_net_used: 0,
            free_net_limit: free_bandwidth,
            net_used: 0,
            net_limit: staked_bandwidth,
            energy_used: 0,
            energy_limit: available_energy,
        }
    }

    #[test]
    fn test_apply_buffer() {
        assert_eq!(apply_buffer(100, 20), 120);
        assert_eq!(apply_buffer(64285, 20), 77142);
        assert_eq!(apply_buffer(1000, 0), 1000);
    }

    #[test]
    fn test_account_usage_bandwidth() {
        let usage = TronAccountUsage {
            free_net_used: 100,
            free_net_limit: 1000,
            net_used: 200,
            net_limit: 500,
            energy_used: 0,
            energy_limit: 0,
        };

        assert_eq!(usage.available_bandwidth(), 1200); // (1000-100) + (500-200)
        assert_eq!(usage.missing_bandwidth(1500), 300);
        assert_eq!(usage.missing_bandwidth(1000), 0);
    }

    #[test]
    fn test_account_usage_energy() {
        let usage = TronAccountUsage {
            free_net_used: 0,
            free_net_limit: 0,
            net_used: 0,
            net_limit: 0,
            energy_used: 10000,
            energy_limit: 60000,
        };

        assert_eq!(usage.available_energy(), 50000);
        assert_eq!(usage.missing_energy(70000), 20000);
        assert_eq!(usage.missing_energy(40000), 0);
    }

    #[test]
    fn test_calculate_token_transfer_fee_no_staked_resources() {
        let usage = account_usage(0, 0, 0);

        let fee = calculate_token_transfer_fee(&usage, 64285, 420, 1000);

        // energy_with_buffer = 64285 * 1.2 = 77142
        // chargeable_energy = 77142 (no staked energy)
        // energy_fee = 77142 * 420 = 32,399,640
        // bandwidth_fee = 345 * 1000 = 345,000
        assert_eq!(fee.fee, 77142 * 420 + DEFAULT_BANDWIDTH_BYTES * 1000);
        assert_eq!(fee.fee_limit, 77142 * 420);
        assert_eq!(fee.energy_price, 420);
    }

    #[test]
    fn test_calculate_token_transfer_fee_with_staked_energy() {
        let usage = account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 60000);

        let fee = calculate_token_transfer_fee(&usage, 64285, 420, 1000);

        // energy_with_buffer = 77142
        // chargeable_energy = 77142 - 60000 = 17142
        // energy_fee = 17142 * 420 = 7,199,640
        // bandwidth_fee = 0 (has enough)
        assert_eq!(fee.fee, 17142 * 420);
        assert_eq!(fee.fee_limit, 77142 * 420);
    }

    #[test]
    fn test_calculate_token_transfer_fee_with_full_coverage() {
        let usage = account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 100000);

        let fee = calculate_token_transfer_fee(&usage, 64285, 420, 1000);

        // User has more than enough staked energy
        assert_eq!(fee.fee, 0);
        assert_eq!(fee.fee_limit, 77142 * 420);
    }

    #[test]
    fn test_calculate_transfer_fee_rate_existing_account() {
        let params = vec![chain_parameter(GET_TRANSACTION_FEE, 1000)];

        let with_bandwidth = account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 0);
        assert_eq!(calculate_transfer_fee_rate(&params, &with_bandwidth, false).unwrap(), BigInt::from(0));

        let without_bandwidth = account_usage(100, 0, 0);
        let expected = BigInt::from(DEFAULT_BANDWIDTH_BYTES * 1000);
        assert_eq!(calculate_transfer_fee_rate(&params, &without_bandwidth, false).unwrap(), expected);
    }

    #[test]
    fn test_calculate_transfer_fee_rate_new_account() {
        let params = vec![
            chain_parameter(GET_TRANSACTION_FEE, 1000),
            chain_parameter(GET_CREATE_ACCOUNT_FEE, 100_000),
            chain_parameter(GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT, 1_000_000),
        ];

        let without_bandwidth = account_usage(0, 0, 0);
        assert_eq!(
            calculate_transfer_fee_rate(&params, &without_bandwidth, true).unwrap(),
            BigInt::from(1_100_000) // activation + bandwidth
        );

        let with_bandwidth = account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 0);
        assert_eq!(
            calculate_transfer_fee_rate(&params, &with_bandwidth, true).unwrap(),
            BigInt::from(1_000_000) // only activation
        );
    }

    #[test]
    fn test_calculate_stake_fee_rate() {
        let params = vec![chain_parameter(GET_TRANSACTION_FEE, 1000)];
        let stake_type = StakeType::Stake(DelegationValidator {
            chain: Chain::Tron,
            id: "validator".to_string(),
            name: "validator".to_string(),
            is_active: true,
            commission: 0.0,
            apr: 0.0,
        });

        let with_bandwidth = account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 0);
        assert_eq!(calculate_stake_fee_rate(&params, &with_bandwidth, &stake_type).unwrap(), BigInt::from(0));

        let without_bandwidth = account_usage(100, 0, 0);
        let expected = BigInt::from(DEFAULT_BANDWIDTH_BYTES * 1000);
        assert_eq!(calculate_stake_fee_rate(&params, &without_bandwidth, &stake_type).unwrap(), expected);
    }

    #[test]
    fn test_get_chain_parameter_value() {
        let params = vec![chain_parameter(GET_ENERGY_FEE, 420), chain_parameter(GET_TRANSACTION_FEE, 1000)];

        assert_eq!(get_chain_parameter_value(&params, GET_ENERGY_FEE).unwrap(), 420);
        assert_eq!(get_chain_parameter_value(&params, GET_TRANSACTION_FEE).unwrap(), 1000);
        assert!(get_chain_parameter_value(&params, "missing").is_err());
    }

    #[test]
    fn test_bandwidth_fee() {
        assert_eq!(bandwidth_fee(&account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 0), DEFAULT_BANDWIDTH_BYTES, 1000), 0);
        assert_eq!(bandwidth_fee(&account_usage(100, 200, 0), DEFAULT_BANDWIDTH_BYTES, 1000), 0);
        assert_eq!(bandwidth_fee(&account_usage(0, 0, 0), DEFAULT_BANDWIDTH_BYTES, 1000), DEFAULT_BANDWIDTH_BYTES * 1000);
        assert_eq!(bandwidth_fee(&account_usage(76, 0, 0), DEFAULT_BANDWIDTH_BYTES, 1000), DEFAULT_BANDWIDTH_BYTES * 1000);
    }

    #[test]
    fn test_calculate_token_transfer_fee_partial_bandwidth() {
        let fee = calculate_token_transfer_fee(&account_usage(76, 0, 0), 64285, 420, 1000);

        assert_eq!(fee.fee, 77142 * 420 + DEFAULT_BANDWIDTH_BYTES * 1000);
        assert_eq!(fee.fee_limit, 77142 * 420);
    }

    #[test]
    fn test_calculate_unfreeze_amounts() {
        let frozen = vec![
            TronFrozen { frozen_type: Some("ENERGY".to_string()), amount: 100 },
            TronFrozen { frozen_type: Some("BANDWIDTH".to_string()), amount: 50 },
        ];

        assert_eq!(calculate_unfreeze_amounts(Some(&frozen), 120), vec![
            TronUnfreeze { resource: Resource::Energy, amount: 100 },
            TronUnfreeze { resource: Resource::Bandwidth, amount: 20 },
        ]);
        assert_eq!(calculate_unfreeze_amounts(Some(&frozen), 50), vec![
            TronUnfreeze { resource: Resource::Energy, amount: 50 },
        ]);
        assert!(calculate_unfreeze_amounts(None, 100).is_empty());
        assert!(calculate_unfreeze_amounts(Some(&frozen), 0).is_empty());
    }
}
