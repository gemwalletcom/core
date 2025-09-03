use std::error::Error;

use num_bigint::BigInt;
use primitives::{fee::FeePriority, fee::GasPriceType, EVMChain, FeeRate, TransactionLoadMetadata};

use crate::fee_calculator::FeeCalculator;
use crate::models::fee::EthereumFeeHistory;

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
}
