use num_bigint::BigInt;
use primitives::{AssetSubtype, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType};
use std::collections::HashMap;

use crate::models::prioritization_fee::SolanaPrioritizationFee;

const STATIC_BASE_FEE: u64 = 5000;

pub fn calculate_transaction_fee(input_type: &TransactionInputType, gas_price_type: &GasPriceType) -> TransactionFee {
    TransactionFee {
        fee: gas_price_type.total_fee(),
        gas_price: gas_price_type.gas_price(),
        gas_limit: get_gas_limit(input_type),
        options: HashMap::new(),
    }
}

pub fn calculate_priority_fee(input_type: &TransactionInputType, prioritization_fees: &[SolanaPrioritizationFee]) -> BigInt {
    let mut fees: Vec<i64> = prioritization_fees.iter().map(|f| f.prioritization_fee).collect();
    fees.sort_by(|a, b| b.cmp(a));
    fees.truncate(5);

    let multiple_of = get_multiple_of(input_type);

    let priority_fee_base = if fees.is_empty() {
        BigInt::from(multiple_of)
    } else {
        let average = fees.iter().sum::<i64>() / fees.len() as i64;
        let rounded = round_to_nearest(average, multiple_of, true);
        BigInt::from(std::cmp::max(rounded, multiple_of))
    };
    priority_fee_base
}

fn get_gas_limit(input_type: &TransactionInputType) -> BigInt {
    match input_type {
        TransactionInputType::Transfer(_) => BigInt::from(100_000),
        TransactionInputType::Swap(_, _) => BigInt::from(420_000),
        TransactionInputType::Stake(_, _) => BigInt::from(100_000),
    }
}

fn get_multiple_of(input_type: &TransactionInputType) -> i64 {
    match input_type {
        TransactionInputType::Transfer(asset) => match &asset.id.token_subtype() {
            AssetSubtype::NATIVE => 25_000,
            AssetSubtype::TOKEN => 50_000,
        },
        TransactionInputType::Stake(_, _) => 25_000,
        TransactionInputType::Swap(_, _) => 100_000,
    }
}

fn round_to_nearest(value: i64, multiple: i64, round_up: bool) -> i64 {
    if round_up {
        ((value + multiple - 1) / multiple) * multiple
    } else {
        (value / multiple) * multiple
    }
}

pub fn calculate_fee_rates(input_type: &TransactionInputType, prioritization_fees: &[SolanaPrioritizationFee]) -> Vec<FeeRate> {
    let mut fees: Vec<i64> = prioritization_fees.iter().map(|f| f.prioritization_fee).collect();
    fees.sort_by(|a, b| b.cmp(a));
    fees.truncate(5);

    let multiple_of = get_multiple_of(input_type);
    let gas_limit = get_gas_limit(input_type);
    let static_base_fee = BigInt::from(STATIC_BASE_FEE);

    let priority_fee_base = if fees.is_empty() {
        BigInt::from(multiple_of)
    } else {
        let average = fees.iter().sum::<i64>() / fees.len() as i64;
        let rounded = round_to_nearest(average, multiple_of, true);
        BigInt::from(std::cmp::max(rounded, multiple_of))
    };

    vec![
        FeeRate::eip1559(
            FeePriority::Slow,
            static_base_fee.clone(),
            (&priority_fee_base / 2_i64 * gas_limit.clone()) / BigInt::from(1_000_000u64),
        ),
        FeeRate::eip1559(
            FeePriority::Normal,
            static_base_fee.clone(),
            (&priority_fee_base * gas_limit.clone()) / BigInt::from(1_000_000u64),
        ),
        FeeRate::eip1559(
            FeePriority::Fast,
            static_base_fee,
            (&priority_fee_base * 3_i64 * gas_limit.clone()) / BigInt::from(1_000_000u64),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, AssetId, AssetType, Chain};

    #[test]
    fn test_calculate_transaction_fee() {
        let gas_price_type = GasPriceType::regular(BigInt::from(1000u64));
        let input_type = TransactionInputType::Transfer(Asset {
            id: AssetId::from_chain(Chain::Solana),
            chain: Chain::Solana,
            token_id: None,
            name: "SOL".to_string(),
            symbol: "SOL".to_string(),
            decimals: 9,
            asset_type: AssetType::NATIVE,
        });

        let fee = calculate_transaction_fee(&input_type, &gas_price_type);
        assert_eq!(fee.fee, BigInt::from(1000u64));
        assert_eq!(fee.gas_price, BigInt::from(1000u64));
        assert_eq!(fee.gas_limit, BigInt::from(100_000u64));
    }

    #[test]
    fn test_calculate_priority_fee() {
        let fees = vec![SolanaPrioritizationFee { prioritization_fee: 150_000 }];
        let input_type = TransactionInputType::Transfer(Asset {
            id: AssetId::from_chain(Chain::Solana),
            chain: Chain::Solana,
            token_id: None,
            name: "SOL".to_string(),
            symbol: "SOL".to_string(),
            decimals: 9,
            asset_type: AssetType::NATIVE,
        });

        let priority_fee = calculate_priority_fee(&input_type, &fees);
        assert_eq!(priority_fee, BigInt::from(150_000));
    }

    #[test]
    fn test_calculate_fee_rates() {
        let fees = vec![SolanaPrioritizationFee { prioritization_fee: 100_000 }];
        let input_type = TransactionInputType::Transfer(Asset {
            id: AssetId::from_chain(Chain::Solana),
            chain: Chain::Solana,
            token_id: None,
            name: "SOL".to_string(),
            symbol: "SOL".to_string(),
            decimals: 9,
            asset_type: AssetType::NATIVE,
        });

        let rates = calculate_fee_rates(&input_type, &fees);
        assert_eq!(rates.len(), 3);
    }
}
