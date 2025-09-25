use std::error::Error;
use std::str::FromStr;

use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use gem_bsc::stake_hub::STAKE_HUB_ADDRESS;
use num_bigint::BigInt;
use num_traits::Num;
use primitives::{
    Chain, EVMChain, FeeRate, StakeType, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, fee::FeePriority,
    fee::GasPriceType,
};

use crate::contracts::IERC20;
use crate::everstake::{DEFAULT_ALLOWED_INTERCHANGE_NUM, EVERSTAKE_ACCOUNTING_ADDRESS, EVERSTAKE_POOL_ADDRESS, EVERSTAKE_SOURCE, IAccounting, IPool};
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
        stake_data: None,
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
        TransactionInputType::TransferNft(_, _) => Err("Unsupported transfer type".into()),
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
            Chain::SmartChain => encode_stake_hub(stake_type, &BigInt::from_str_radix(&input.value, 10)?),
            Chain::Ethereum => encode_everstake(stake_type, &BigInt::from_str_radix(&input.value, 10)?),
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
        TransactionInputType::TransferNft(_, _) => Err("Unsupported transfer type".into()),
        TransactionInputType::TokenApprove(_, approval) => Ok(approval.token.clone()),
        TransactionInputType::Generic(_, _, _) => Ok(input.destination_address.clone()),
        TransactionInputType::Stake(_, stake_type) => match chain.to_chain() {
            Chain::SmartChain => Ok(STAKE_HUB_ADDRESS.to_string()),
            Chain::Ethereum => match stake_type {
                StakeType::Stake(_) | StakeType::Unstake(_) => Ok(EVERSTAKE_POOL_ADDRESS.to_string()),
                StakeType::Withdraw(_) => Ok(EVERSTAKE_ACCOUNTING_ADDRESS.to_string()),
                _ => Err("Unsupported stake type".into()),
            },
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
        TransactionInputType::TransferNft(_, _) => Ok(BigInt::from(0)),
        TransactionInputType::TokenApprove(_, _) => Ok(BigInt::from(0)),
        TransactionInputType::Generic(_, _, _) => Ok(value),
        TransactionInputType::Stake(_, stake_type) => match chain.to_chain() {
            Chain::SmartChain => match stake_type {
                StakeType::Stake(_) => Ok(value),
                StakeType::Unstake(_) | StakeType::Redelegate(_) | StakeType::Withdraw(_) => Ok(BigInt::from(0)),
                _ => Ok(BigInt::from(0)),
            },
            Chain::Ethereum => match stake_type {
                StakeType::Stake(_) => Ok(value),
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
        TransactionInputType::Transfer(asset)
        | TransactionInputType::Deposit(asset)
        | TransactionInputType::TransferNft(asset, _)
        | TransactionInputType::Account(asset, _) => {
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

fn big_int_to_u256(value: &BigInt) -> Result<U256, Box<dyn Error + Send + Sync>> {
    if value < &BigInt::from(0) {
        return Err("Negative values are not supported".into());
    }

    U256::from_str(&value.to_string()).map_err(|e| e.to_string().into())
}

fn encode_everstake(stake_type: &StakeType, amount: &BigInt) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    match stake_type {
        StakeType::Stake(_) => Ok(IPool::stakeCall { source: EVERSTAKE_SOURCE }.abi_encode()),
        StakeType::Unstake(_) => {
            let value = big_int_to_u256(amount)?;
            let call = IPool::unstakeCall {
                value,
                allowedInterchangeNum: DEFAULT_ALLOWED_INTERCHANGE_NUM,
                source: EVERSTAKE_SOURCE,
            };
            Ok(call.abi_encode())
        }
        StakeType::Withdraw(_) => Ok(IAccounting::claimWithdrawRequestCall {}.abi_encode()),
        _ => Err("Unsupported stake type for Everstake".into()),
    }
}

fn encode_stake_hub(stake_type: &StakeType, amount: &BigInt) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    match stake_type {
        StakeType::Stake(validator) => gem_bsc::stake_hub::encode_delegate_call(&validator.id, false).map_err(|e| e.to_string().into()),
        StakeType::Unstake(delegation) => {
            // Calculate shares based on amount and delegation balance/shares ratio
            let amount_shares = amount * &delegation.base.shares / &delegation.base.balance;

            gem_bsc::stake_hub::encode_undelegate_call(&delegation.validator.id, &amount_shares.to_string()).map_err(|e| e.to_string().into())
        }
        StakeType::Redelegate(redelegate_data) => {
            // Calculate shares based on amount and delegation balance/shares ratio
            let amount_shares = amount * &redelegate_data.delegation.base.shares / &redelegate_data.delegation.base.balance;

            gem_bsc::stake_hub::encode_redelegate_call(
                &redelegate_data.delegation.validator.id,
                &redelegate_data.to_validator.id,
                &amount_shares.to_string(),
                false,
            )
            .map_err(|e| e.to_string().into())
        }
        StakeType::Withdraw(delegation) => {
            // Request number 0 means claim all
            gem_bsc::stake_hub::encode_claim_call(&delegation.validator.id, 0).map_err(|e| e.to_string().into())
        }
        _ => Err("Unsupported stake type for StakeHub".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::everstake::{EVERSTAKE_POOL_ADDRESS, IAccounting};
    use primitives::{Delegation, DelegationBase, DelegationState, DelegationValidator, RedelegateData};

    fn everstake_validator() -> DelegationValidator {
        DelegationValidator {
            chain: Chain::Ethereum,
            id: EVERSTAKE_POOL_ADDRESS.to_string(),
            name: "Everstake Pool".to_string(),
            is_active: true,
            commission: 10.0,
            apr: 4.2,
        }
    }

    #[test]
    fn test_map_transaction_preload_with_hex_prefix() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let nonce_hex = "0xa".to_string();
        let chain_id = "1".to_string();

        let result = map_transaction_preload(nonce_hex, chain_id)?;

        match result {
            TransactionLoadMetadata::Evm { nonce, chain_id, stake_data } => {
                assert_eq!(nonce, 10);
                assert_eq!(chain_id, 1);
                assert!(stake_data.is_none());
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

    #[test]
    fn test_encode_stake_hub_delegate() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let validator = DelegationValidator {
            chain: Chain::SmartChain,
            id: "0x773760b0708a5Cc369c346993a0c225D8e4043B1".to_string(),
            name: "Test Validator".to_string(),
            is_active: true,
            commission: 5.0,
            apr: 10.0,
        };

        let stake_type = StakeType::Stake(validator);
        let amount = BigInt::from(1_000_000_000_000_000_000u64); // 1 BNB

        let result = encode_stake_hub(&stake_type, &amount)?;

        // Should encode a delegate call
        assert!(!result.is_empty());
        // The first 4 bytes should be the function selector for delegate
        let selector = &result[0..4];
        assert_eq!(hex::encode(selector), "982ef0a7");

        Ok(())
    }

    #[test]
    fn test_encode_stake_hub_unstake() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let delegation = Delegation {
            base: DelegationBase {
                asset_id: primitives::AssetId::from_chain(Chain::SmartChain),
                state: DelegationState::Active,
                balance: BigInt::from(2_000_000_000_000_000_000u64), // 2 BNB
                shares: BigInt::from(1_900_000_000_000_000_000u64),  // Slightly less shares
                rewards: BigInt::from(0),
                completion_date: None,
                delegation_id: "test".to_string(),
                validator_id: "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A".to_string(),
            },
            validator: DelegationValidator {
                chain: Chain::SmartChain,
                id: "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A".to_string(),
                name: "Test Validator".to_string(),
                is_active: true,
                commission: 5.0,
                apr: 10.0,
            },
            price: None,
        };

        let stake_type = StakeType::Unstake(delegation);
        let amount = BigInt::from(1_000_000_000_000_000_000u64); // Unstake 1 BNB

        let result = encode_stake_hub(&stake_type, &amount)?;

        assert!(!result.is_empty());
        // The first 4 bytes should be the function selector for undelegate
        let selector = &result[0..4];
        assert_eq!(hex::encode(selector), "4d99dd16");

        Ok(())
    }

    #[test]
    fn test_encode_stake_hub_redelegate() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let delegation = Delegation {
            base: DelegationBase {
                asset_id: primitives::AssetId::from_chain(Chain::SmartChain),
                state: DelegationState::Active,
                balance: BigInt::from(2_000_000_000_000_000_000u64), // 2 BNB
                shares: BigInt::from(1_900_000_000_000_000_000u64),  // Slightly less shares
                rewards: BigInt::from(0),
                completion_date: None,
                delegation_id: "test".to_string(),
                validator_id: "0x773760b0708a5Cc369c346993a0c225D8e4043B1".to_string(),
            },
            validator: DelegationValidator {
                chain: Chain::SmartChain,
                id: "0x773760b0708a5Cc369c346993a0c225D8e4043B1".to_string(),
                name: "Source Validator".to_string(),
                is_active: true,
                commission: 5.0,
                apr: 10.0,
            },
            price: None,
        };

        let to_validator = DelegationValidator {
            chain: Chain::SmartChain,
            id: "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A".to_string(),
            name: "Target Validator".to_string(),
            is_active: true,
            commission: 3.0,
            apr: 12.0,
        };

        let redelegate_data = RedelegateData { delegation, to_validator };

        let stake_type = StakeType::Redelegate(redelegate_data);
        let amount = BigInt::from(1_000_000_000_000_000_000u64); // Redelegate 1 BNB

        let result = encode_stake_hub(&stake_type, &amount)?;

        assert!(!result.is_empty());
        // The first 4 bytes should be the function selector for redelegate
        let selector = &result[0..4];
        assert_eq!(hex::encode(selector), "59491871");

        Ok(())
    }

    #[test]
    fn test_encode_stake_hub_withdraw() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let delegation = Delegation {
            base: DelegationBase {
                asset_id: primitives::AssetId::from_chain(Chain::SmartChain),
                state: DelegationState::AwaitingWithdrawal,
                balance: BigInt::from(1_000_000_000_000_000_000u64),
                shares: BigInt::from(1_000_000_000_000_000_000u64),
                rewards: BigInt::from(0),
                completion_date: None,
                delegation_id: "test".to_string(),
                validator_id: "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A".to_string(),
            },
            validator: DelegationValidator {
                chain: Chain::SmartChain,
                id: "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A".to_string(),
                name: "Test Validator".to_string(),
                is_active: true,
                commission: 5.0,
                apr: 10.0,
            },
            price: None,
        };

        let stake_type = StakeType::Withdraw(delegation);
        let amount = BigInt::from(0); // Amount doesn't matter for withdraw

        let result = encode_stake_hub(&stake_type, &amount)?;

        assert!(!result.is_empty());
        // The first 4 bytes should be the function selector for claim
        let selector = &result[0..4];
        assert_eq!(hex::encode(selector), "aad3ec96");

        Ok(())
    }

    #[test]
    fn test_encode_everstake_stake() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let stake_type = StakeType::Stake(everstake_validator());
        let amount = BigInt::from(1_000_000_000_000_000_000u64);

        let result = encode_everstake(&stake_type, &amount)?;

        let expected_hex = "3a29dbae0000000000000000000000000000000000000000000000000000000000000017";
        assert_eq!(hex::encode(&result), expected_hex);

        Ok(())
    }

    #[test]
    fn test_encode_everstake_unstake() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let validator = everstake_validator();
        let delegation = Delegation {
            base: DelegationBase {
                asset_id: primitives::AssetId::from_chain(Chain::Ethereum),
                state: DelegationState::Active,
                balance: BigInt::from(2_000_000_000_000_000_000u64),
                shares: BigInt::from(0),
                rewards: BigInt::from(0),
                completion_date: None,
                delegation_id: "eth-delegation".to_string(),
                validator_id: EVERSTAKE_POOL_ADDRESS.to_string(),
            },
            validator: validator.clone(),
            price: None,
        };

        let stake_type = StakeType::Unstake(delegation);
        let amount = BigInt::from(500_000_000_000_000_000u64);

        let result = encode_everstake(&stake_type, &amount)?;

        let expected_hex = "76ec871c00000000000000000000000000000000000000000000000006f05b59d3b2000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000017";
        assert_eq!(hex::encode(&result), expected_hex);

        Ok(())
    }

    #[test]
    fn test_encode_everstake_withdraw() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let validator = everstake_validator();
        let delegation = Delegation {
            base: DelegationBase {
                asset_id: primitives::AssetId::from_chain(Chain::Ethereum),
                state: DelegationState::AwaitingWithdrawal,
                balance: BigInt::from(750_000_000_000_000_000u64),
                shares: BigInt::from(0),
                rewards: BigInt::from(0),
                completion_date: None,
                delegation_id: "eth-withdraw".to_string(),
                validator_id: EVERSTAKE_POOL_ADDRESS.to_string(),
            },
            validator,
            price: None,
        };

        let stake_type = StakeType::Withdraw(delegation);
        let result = encode_everstake(&stake_type, &BigInt::from(0))?;

        let expected_hex = "33986ffa";
        assert_eq!(hex::encode(&result), expected_hex);
        assert_eq!(result, IAccounting::claimWithdrawRequestCall {}.abi_encode());

        Ok(())
    }
}
