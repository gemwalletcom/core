use num_bigint::BigInt;
use primitives::{AssetSubtype, Chain, FeeOption, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType};
use std::collections::HashMap;

use crate::models::jito::JITO_TIP_MIN_LAMPORTS;
use crate::{constants::STATIC_BASE_FEE, models::prioritization_fee::SolanaPrioritizationFee};

pub fn calculate_transaction_fee(input_type: &TransactionInputType, gas_price_type: &GasPriceType, recipient_token_address: Option<String>) -> TransactionFee {
    let mut options = HashMap::new();
    let recipient_asset = input_type.get_recipient_asset();
    if recipient_asset.chain() == Chain::Solana && recipient_asset.id.token_subtype() == AssetSubtype::TOKEN && recipient_token_address.is_none() {
        options.insert(
            FeeOption::TokenAccountCreation,
            BigInt::from(input_type.get_asset().id.chain.token_activation_fee().unwrap_or(0)),
        );
    }
    TransactionFee::new_gas_price_type(gas_price_type.clone(), gas_price_type.total_fee(), get_gas_limit(input_type), options)
}

pub fn calculate_priority_fee(input_type: &TransactionInputType, prioritization_fees: &[SolanaPrioritizationFee]) -> BigInt {
    let mut fees: Vec<i64> = prioritization_fees.iter().map(|f| f.prioritization_fee).collect();
    fees.sort_by(|a, b| b.cmp(a));
    fees.truncate(5);

    let multiple_of = get_multiple_of(input_type);

    if fees.is_empty() {
        BigInt::from(multiple_of)
    } else {
        let average = fees.iter().sum::<i64>() / fees.len() as i64;
        let rounded = round_to_nearest(average, multiple_of, true);
        BigInt::from(std::cmp::max(rounded, multiple_of))
    }
}

fn get_gas_limit(input_type: &TransactionInputType) -> BigInt {
    match input_type {
        TransactionInputType::Transfer(_)
        | TransactionInputType::Deposit(_)
        | TransactionInputType::TransferNft(_, _)
        | TransactionInputType::Account(_, _)
        | TransactionInputType::TokenApprove(_, _)
        | TransactionInputType::Generic(_, _, _)
        | TransactionInputType::Perpetual(_, _)
        | TransactionInputType::Earn(_, _, _) => BigInt::from(100_000),
        TransactionInputType::Swap(_, _, _) => BigInt::from(420_000),
        TransactionInputType::Stake(_, _) => BigInt::from(100_000),
    }
}

fn get_multiple_of(input_type: &TransactionInputType) -> i64 {
    match input_type {
        TransactionInputType::Transfer(asset)
        | TransactionInputType::Deposit(asset)
        | TransactionInputType::TransferNft(asset, _)
        | TransactionInputType::Account(asset, _)
        | TransactionInputType::TokenApprove(asset, _)
        | TransactionInputType::Generic(asset, _, _)
        | TransactionInputType::Perpetual(asset, _)
        | TransactionInputType::Earn(asset, _, _) => match &asset.id.token_subtype() {
            AssetSubtype::NATIVE => 25_000,
            AssetSubtype::TOKEN => 50_000,
        },
        TransactionInputType::Stake(_, _) => 25_000,
        TransactionInputType::Swap(_, _, _) => 100_000,
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
    let static_base_fee = BigInt::from(STATIC_BASE_FEE);

    let total_priority_base = if fees.is_empty() {
        BigInt::from(multiple_of)
    } else {
        let average = fees.iter().sum::<i64>() / fees.len() as i64;
        let rounded = round_to_nearest(average, multiple_of, true);
        BigInt::from(std::cmp::max(rounded, multiple_of))
    };

    let gas_limit = get_gas_limit(input_type);

    [FeePriority::Slow, FeePriority::Normal, FeePriority::Fast]
        .iter()
        .map(|priority| {
            let total_priority = match priority {
                FeePriority::Slow => &total_priority_base / 2,
                FeePriority::Normal => total_priority_base.clone(),
                FeePriority::Fast => &total_priority_base * 3,
            };

            // 70% priority fee, 30% jito tip https://docs.jito.wtf/lowlatencytxnsend/#id18
            let scaled_total = (total_priority.clone() * gas_limit.clone()) / BigInt::from(1_000_000);
            let priority_fee = (&scaled_total * 70) / 100;
            let jito_tip_calculated: BigInt = (&scaled_total * 30) / 100;
            let unit_price = (&total_priority * 70) / 100;
            let jito_tip = jito_tip_calculated.to_u64_digits().1.first().copied().unwrap_or(0).max(JITO_TIP_MIN_LAMPORTS);

            FeeRate::new(*priority, GasPriceType::solana(static_base_fee.clone(), priority_fee, unit_price, jito_tip))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::swap::SwapData;
    use primitives::{Asset, AssetId, AssetType, Chain, SwapProvider};

    #[test]
    fn test_calculate_transaction_fee() {
        // Test with EIP-1559 gas pricing
        let gas_price_type = GasPriceType::eip1559(BigInt::from(5000u64), BigInt::from(15000u64));
        let input_type = TransactionInputType::Transfer(Asset {
            id: AssetId::from_chain(Chain::Solana),
            chain: Chain::Solana,
            token_id: None,
            name: "SOL".to_string(),
            symbol: "SOL".to_string(),
            decimals: 9,
            asset_type: AssetType::NATIVE,
        });

        let fee = calculate_transaction_fee(&input_type, &gas_price_type, None);

        assert_eq!(fee.fee, BigInt::from(20_000u64));
        assert_eq!(fee.gas_price_type.gas_price(), BigInt::from(5000u64));
        assert_eq!(fee.gas_price_type.priority_fee(), BigInt::from(15000u64));
        assert_eq!(fee.gas_limit, BigInt::from(100_000u64));
        assert!(fee.options.is_empty());
    }

    #[test]
    fn test_calculate_transaction_fee_swap() {
        // Test swap transaction with higher gas limit
        let gas_price_type = GasPriceType::eip1559(BigInt::from(5000u64), BigInt::from(30000u64));
        let input_type = TransactionInputType::Swap(
            Asset {
                id: AssetId::from_chain(Chain::Solana),
                chain: Chain::Solana,
                token_id: None,
                name: "SOL".to_string(),
                symbol: "SOL".to_string(),
                decimals: 9,
                asset_type: AssetType::NATIVE,
            },
            Asset {
                id: AssetId::from_chain(Chain::Solana),
                chain: Chain::Solana,
                token_id: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
                name: "USDC".to_string(),
                symbol: "USDC".to_string(),
                decimals: 6,
                asset_type: AssetType::SPL,
            },
            SwapData::mock_with_provider(SwapProvider::Jupiter),
        );

        let fee = calculate_transaction_fee(&input_type, &gas_price_type, None);

        assert_eq!(fee.fee, BigInt::from(35_000u64));
        assert_eq!(fee.gas_limit, BigInt::from(420_000u64));
    }

    #[test]
    fn test_calculate_transaction_fee_cross_chain_swap_without_token_creation() {
        let gas_price_type = GasPriceType::eip1559(BigInt::from(5000u64), BigInt::from(15000u64));
        let input_type = TransactionInputType::Swap(Asset::mock_spl_token(), Asset::mock_ethereum_usdc(), SwapData::mock_with_provider(SwapProvider::Jupiter));

        let fee = calculate_transaction_fee(&input_type, &gas_price_type, None);

        assert!(!fee.options.contains_key(&FeeOption::TokenAccountCreation));
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
        let fees = vec![SolanaPrioritizationFee { prioritization_fee: 25_000 }];
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

        for rate in &rates {
            assert_eq!(rate.gas_price_type.gas_price(), BigInt::from(5000u64));
        }

        assert_eq!(rates[0].priority, FeePriority::Slow);
        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(875));
        assert_eq!(rates[0].gas_price_type.unit_price(), BigInt::from(8_750));
        assert_eq!(rates[0].gas_price_type.jito_tip(), 10_000); // min jito tip

        assert_eq!(rates[1].priority, FeePriority::Normal);
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(1_750));
        assert_eq!(rates[1].gas_price_type.unit_price(), BigInt::from(17_500));
        assert_eq!(rates[1].gas_price_type.jito_tip(), 10_000); // min jito tip

        assert_eq!(rates[2].priority, FeePriority::Fast);
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(5_250));
        assert_eq!(rates[2].gas_price_type.unit_price(), BigInt::from(52_500));
        assert_eq!(rates[2].gas_price_type.jito_tip(), 10_000); // min jito tip
    }

    #[test]
    fn test_calculate_fee_rates_empty_fees() {
        let fees = vec![];
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
        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(875u64));
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(1_750u64));
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(5_250u64));
    }

    #[test]
    fn test_calculate_fee_rates_spl_token() {
        let fees = vec![SolanaPrioritizationFee { prioritization_fee: 80_000 }];
        let input_type = TransactionInputType::Transfer(Asset {
            id: AssetId::from_chain(Chain::Solana),
            chain: Chain::Solana,
            token_id: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
            name: "USDC".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
            asset_type: AssetType::SPL,
        });

        let rates = calculate_fee_rates(&input_type, &fees);
        assert_eq!(rates.len(), 3);

        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(3_500u64));
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(7_000u64));
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(21_000u64));
    }

    #[test]
    fn test_calculate_fee_rates_swap() {
        let fees = vec![SolanaPrioritizationFee { prioritization_fee: 150_000 }];
        let input_type = TransactionInputType::Swap(
            Asset {
                id: AssetId::from_chain(Chain::Solana),
                chain: Chain::Solana,
                token_id: None,
                name: "SOL".to_string(),
                symbol: "SOL".to_string(),
                decimals: 9,
                asset_type: AssetType::NATIVE,
            },
            Asset {
                id: AssetId::from_chain(Chain::Solana),
                chain: Chain::Solana,
                token_id: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
                name: "USDC".to_string(),
                symbol: "USDC".to_string(),
                decimals: 6,
                asset_type: AssetType::SPL,
            },
            SwapData::mock_with_provider(SwapProvider::Jupiter),
        );

        let rates = calculate_fee_rates(&input_type, &fees);
        assert_eq!(rates.len(), 3);

        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(29_400u64));
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(58_800u64));
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(176_400u64));
    }

    #[test]
    fn test_calculate_fee_rates_multiple_fees() {
        let fees = vec![
            SolanaPrioritizationFee { prioritization_fee: 200_000 },
            SolanaPrioritizationFee { prioritization_fee: 150_000 },
            SolanaPrioritizationFee { prioritization_fee: 175_000 },
            SolanaPrioritizationFee { prioritization_fee: 125_000 },
            SolanaPrioritizationFee { prioritization_fee: 225_000 },
            SolanaPrioritizationFee { prioritization_fee: 100_000 }, // Should be truncated (6th fee)
        ];
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

        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(6_125u64));
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(12_250u64));
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(36_750u64));
    }

    #[test]
    fn test_fee_calculation_matches_swift() {
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

        let rates = calculate_fee_rates(&input_type, &fees);

        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(5_250));
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(10_500));
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(31_500));
    }

    #[test]
    fn test_calculate_transaction_fee_token_recipient_exists() {
        let gas_price_type = GasPriceType::eip1559(BigInt::from(5000u64), BigInt::from(15000u64));
        let asset = Asset {
            id: AssetId::new("solana_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
            chain: Chain::Solana,
            token_id: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
            name: "USDC".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
            asset_type: AssetType::SPL,
        };
        let input_type = TransactionInputType::Transfer(asset);

        let fee = calculate_transaction_fee(&input_type, &gas_price_type, Some("existing_account".to_string()));

        assert_eq!(fee.fee, BigInt::from(20_000u64));
        assert!(fee.options.is_empty());
    }

    #[test]
    fn test_calculate_transaction_fee_token_recipient_new() {
        let gas_price_type = GasPriceType::eip1559(BigInt::from(5000u64), BigInt::from(15000u64));
        let asset = Asset {
            id: AssetId::new("solana_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
            chain: Chain::Solana,
            token_id: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
            name: "USDC".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
            asset_type: AssetType::SPL,
        };
        let input_type = TransactionInputType::Transfer(asset);

        let fee = calculate_transaction_fee(&input_type, &gas_price_type, None);

        assert_eq!(fee.fee, BigInt::from(2_059_280u64)); // 20_000 gas + 2_039_280 token account creation
        assert_eq!(fee.options.len(), 1);
        assert!(fee.options.contains_key(&FeeOption::TokenAccountCreation));
        assert_eq!(fee.options[&FeeOption::TokenAccountCreation], BigInt::from(2_039_280u64));
    }
}
