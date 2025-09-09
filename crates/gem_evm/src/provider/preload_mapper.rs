use std::error::Error;
use std::str::FromStr;

use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use num_bigint::BigInt;
use num_traits::Num;
use primitives::{
    fee::FeePriority, fee::GasPriceType, Chain, EVMChain, FeeRate, StakeType, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata,
};

use crate::contracts::IERC20;
use crate::fee_calculator::FeeCalculator;
use crate::models::fee::EthereumFeeHistory;

const GAS_LIMIT_PERCENT_INCREASE: u32 = 50;
const GAS_LIMIT_21000: u64 = 21000;

pub fn bigint_to_hex_string(value: &BigInt) -> String {
    format!("0x{:x}", value)
}

pub fn bytes_to_hex_string(data: &[u8]) -> String {
    format!("0x{}", alloy_primitives::hex::encode(data))
}

pub fn map_transaction_preload(nonce_hex: String, chain_id: String) -> Result<TransactionLoadMetadata, Box<dyn std::error::Error + Send + Sync>> {
    let nonce = u64::from_str_radix(nonce_hex.trim_start_matches("0x"), 16)?;
    Ok(TransactionLoadMetadata::Evm {
        nonce,
        chain_id: chain_id.parse::<u64>()?,
    })
}

pub fn map_transaction_fee_rates(chain: EVMChain, fee_history: &EthereumFeeHistory) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
    let base_fee = fee_history.base_fee_per_gas.last().ok_or("No base fee available")?;
    let min_priority_fee = BigInt::from(chain.min_priority_fee());

    Ok(FeeCalculator::new()
        .calculate_priority_fees(
            fee_history,
            &[FeePriority::Slow, FeePriority::Normal, FeePriority::Fast],
            min_priority_fee.clone(),
        )?
        .into_iter()
        .map(|x| {
            let priority_fee = BigInt::max(min_priority_fee.clone(), x.value.clone());
            FeeRate::new(x.priority, GasPriceType::eip1559(base_fee.clone(), priority_fee))
        })
        .collect())
}

pub fn get_transaction_data(chain: EVMChain, input: &TransactionLoadInput) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    match &input.input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => {
            if asset.id.is_native() {
                Ok(vec![])
            } else {
                let value = BigInt::from_str_radix(&input.value, 10)?;
                Ok(encode_erc20_transfer(&input.destination_address, &value)?)
            }
        }
        TransactionInputType::Swap(_, _, swap_data) => {
            if let Some(approval) = &swap_data.data.approval {
                Ok(encode_erc20_approve(&approval.spender)?)
            } else {
                Ok(alloy_primitives::hex::decode(&swap_data.data.data)?)
            }
        }
        TransactionInputType::TokenApprove(_, approval) => Ok(encode_erc20_approve(&approval.spender)?),
        TransactionInputType::Generic(_, _, extra) => Ok(extra.data.clone().unwrap_or_default()),
        TransactionInputType::Stake(_, stake_type) => match chain.to_chain() {
            Chain::SmartChain => {
                let value = BigInt::from_str_radix(&input.value, 10)?;
                Ok(encode_stake_hub(stake_type, &input.destination_address, &value)?)
            }
            _ => Err("Unsupported chain for staking".into()),
        },
        _ => Err("Unsupported transfer type".into()),
    }
}

pub fn get_transaction_to(chain: EVMChain, input: &TransactionLoadInput) -> Result<String, Box<dyn Error + Send + Sync>> {
    match &input.input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => {
            if asset.id.is_native() {
                Ok(input.destination_address.clone())
            } else {
                Ok(asset.token_id.as_ref().ok_or("Missing token ID")?.clone())
            }
        }
        TransactionInputType::Swap(_, _, swap_data) => {
            if let Some(approval) = &swap_data.data.approval {
                Ok(approval.token.clone())
            } else {
                Ok(input.destination_address.clone())
            }
        }
        TransactionInputType::TokenApprove(_, approval) => Ok(approval.token.clone()),
        TransactionInputType::Generic(_, _, _) => Ok(input.destination_address.clone()),
        TransactionInputType::Stake(_, _) => match chain.to_chain() {
            Chain::SmartChain => Ok("0x0000000000000000000000000000000000002002".to_string()),
            Chain::Ethereum => Ok(input.destination_address.clone()),
            _ => Err("Unsupported chain for staking".into()),
        },
        _ => Err("Unsupported transfer type".into()),
    }
}

pub fn get_transaction_value(chain: EVMChain, input: &TransactionLoadInput) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let value = BigInt::from_str_radix(&input.value, 10)?;

    match &input.input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => {
            if asset.id.is_native() {
                Ok(value)
            } else {
                Ok(BigInt::from(0))
            }
        }
        TransactionInputType::Swap(_, _, swap_data) => {
            if swap_data.data.approval.is_some() {
                Ok(BigInt::from(0))
            } else {
                BigInt::from_str_radix(&swap_data.data.value, 10).map_err(|e| e.to_string().into())
            }
        }
        TransactionInputType::TokenApprove(_, _) => Ok(BigInt::from(0)),
        TransactionInputType::Generic(_, _, _) => Ok(value),
        TransactionInputType::Stake(_, stake_type) => match chain.to_chain() {
            Chain::SmartChain | Chain::Ethereum => match stake_type {
                StakeType::Stake(_) => Ok(value),
                StakeType::Unstake(_) | StakeType::Redelegate(_) | StakeType::Withdraw(_) => Ok(BigInt::from(0)),
                _ => Ok(BigInt::from(0)),
            },
            _ => Ok(BigInt::from(0)),
        },
        _ => Ok(BigInt::from(0)),
    }
}

pub fn calculate_gas_limit_with_increase(gas_limit: BigInt) -> BigInt {
    if gas_limit == BigInt::from(GAS_LIMIT_21000) {
        gas_limit
    } else {
        gas_limit * BigInt::from(100 + GAS_LIMIT_PERCENT_INCREASE) / BigInt::from(100)
    }
}

pub fn get_priority_fee_by_type(input_type: &TransactionInputType, is_max_value: bool, gas_price_type: &GasPriceType) -> BigInt {
    match input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => {
            if asset.id.is_native() && is_max_value {
                gas_price_type.gas_price()
            } else {
                gas_price_type.priority_fee()
            }
        }
        _ => gas_price_type.priority_fee(),
    }
}

pub fn get_extra_fee_gas_limit(input: &TransactionLoadInput) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    match &input.input_type {
        TransactionInputType::Swap(_, _, swap_data) => {
            if swap_data.data.approval.is_some() {
                if let Some(ref gas_limit) = swap_data.data.gas_limit {
                    Ok(BigInt::from_str_radix(gas_limit, 10)?)
                } else {
                    Ok(BigInt::from(0))
                }
            } else {
                Ok(BigInt::from(0))
            }
        }
        _ => Ok(BigInt::from(0)),
    }
}

fn encode_erc20_transfer(to: &str, amount: &BigInt) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let to_address = Address::from_str(to)?;
    let value = U256::from_str(&amount.to_string())?;
    let call = IERC20::transferCall { to: to_address, value };
    Ok(call.abi_encode())
}

fn encode_erc20_approve(spender: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let spender_address = Address::from_str(spender)?;
    let max_value = U256::MAX;
    let call = IERC20::approveCall {
        spender: spender_address,
        value: max_value,
    };
    Ok(call.abi_encode())
}

fn encode_stake_hub(stake_type: &StakeType, validator_address: &str, _amount: &BigInt) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    match stake_type {
        StakeType::Stake(_) => gem_bsc::stake_hub::encode_delegate_call(validator_address, false).map_err(|e| e.to_string().into()),
        StakeType::Unstake(delegation) => {
            gem_bsc::stake_hub::encode_undelegate_call(&delegation.base.validator_id, &delegation.base.shares.to_string()).map_err(|e| e.to_string().into())
        }
        StakeType::Redelegate(redelegate_data) => gem_bsc::stake_hub::encode_redelegate_call(
            &redelegate_data.delegation.base.validator_id,
            &redelegate_data.to_validator.id,
            &redelegate_data.delegation.base.shares.to_string(),
            false,
        )
        .map_err(|e| e.to_string().into()),
        StakeType::Withdraw(delegation) => gem_bsc::stake_hub::encode_claim_call(&delegation.base.validator_id, 0).map_err(|e| e.to_string().into()),
        _ => Err("Unsupported stake type for StakeHub".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_transaction_preload_with_hex_prefix() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let nonce_hex = "0xa".to_string();
        let chain_id = "1".to_string();

        let result = map_transaction_preload(nonce_hex, chain_id)?;

        match result {
            TransactionLoadMetadata::Evm { nonce, chain_id } => {
                assert_eq!(nonce, 10);
                assert_eq!(chain_id, 1);
            }
            _ => panic!("Expected Evm variant"),
        }

        Ok(())
    }

    #[test]
    fn test_map_transaction_preload_invalid_nonce() {
        let nonce_hex = "invalid".to_string();
        let chain_id_hex = "0x1".to_string();

        let result = map_transaction_preload(nonce_hex, chain_id_hex);

        assert!(result.is_err());
    }

    #[test]
    fn test_map_transaction_preload_invalid_chain_id() {
        let nonce_hex = "0x1".to_string();
        let chain_id_hex = "invalid".to_string();

        let result = map_transaction_preload(nonce_hex, chain_id_hex);

        assert!(result.is_err());
    }

    fn create_test_fee_history_for_mapper() -> EthereumFeeHistory {
        EthereumFeeHistory {
            reward: vec![vec!["0x5f5e100".to_string(), "0xbebc200".to_string(), "0x11e1a300".to_string()]],
            base_fee_per_gas: vec![BigInt::from(20_000_000_000u64)],
            gas_used_ratio: vec![0.5],
            oldest_block: 0x1234,
        }
    }

    #[test]
    fn test_map_transaction_fee_rates_normal_case() -> Result<(), Box<dyn Error + Sync + Send>> {
        let fee_history = create_test_fee_history_for_mapper();

        let result = map_transaction_fee_rates(EVMChain::Ethereum, &fee_history)?;

        assert_eq!(result.len(), 3);

        let min_priority_fee = BigInt::from(EVMChain::Ethereum.min_priority_fee());
        for fee_rate in &result {
            match &fee_rate.gas_price_type {
                GasPriceType::Eip1559 { gas_price, priority_fee } => {
                    assert!(*gas_price >= min_priority_fee);
                    assert!(*priority_fee >= min_priority_fee);
                }
                _ => panic!("Expected EIP-1559 gas price type"),
            }
        }

        Ok(())
    }

    #[test]
    fn test_map_transaction_fee_rates_zero_base_fee() -> Result<(), Box<dyn Error + Sync + Send>> {
        let fee_history = EthereumFeeHistory {
            reward: vec![vec!["0x5f5e100".to_string(), "0xbebc200".to_string(), "0x11e1a300".to_string()]],
            base_fee_per_gas: vec![BigInt::from(0u64)], // Zero base fee
            gas_used_ratio: vec![0.5],
            oldest_block: 0x1234,
        };

        let result = map_transaction_fee_rates(EVMChain::SmartChain, &fee_history)?;

        assert_eq!(result.len(), 3);

        assert_eq!(result[0].gas_price_type.gas_price(), BigInt::ZERO);
        assert!(result[0].gas_price_type.priority_fee() != BigInt::ZERO);

        Ok(())
    }

    #[test]
    fn test_map_transaction_fee_rates_invalid_hex() {
        let fee_history = EthereumFeeHistory {
            reward: vec![vec!["invalid_hex".to_string()]],
            base_fee_per_gas: vec![BigInt::from(20_000_000_000u64)],
            gas_used_ratio: vec![0.5],
            oldest_block: 0x1234,
        };

        let result = map_transaction_fee_rates(EVMChain::Ethereum, &fee_history);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_gas_limit_with_increase() {
        let gas_21000 = BigInt::from(21000);
        let result = calculate_gas_limit_with_increase(gas_21000.clone());
        assert_eq!(result, gas_21000);

        let gas_100000 = BigInt::from(100000);
        let result = calculate_gas_limit_with_increase(gas_100000);
        assert_eq!(result, BigInt::from(150000));
    }

    #[test]
    fn test_bigint_to_string_conversion() {
        let value = BigInt::from(100_000_000u64);
        assert_eq!(value.to_string(), "100000000");

        let min_priority = BigInt::from(primitives::EVMChain::Ethereum.min_priority_fee());
        assert_eq!(min_priority.to_string(), "100000000");
    }
}
