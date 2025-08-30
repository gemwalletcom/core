use num_bigint::BigInt;
use primitives::{AssetSubtype, FeeOption, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType};
use std::collections::HashMap;

use crate::{constants::STATIC_BASE_FEE, models::prioritization_fee::SolanaPrioritizationFee};

pub fn calculate_transaction_fee(input_type: &TransactionInputType, gas_price_type: &GasPriceType, recipient_token_address: Option<String>) -> TransactionFee {
    let mut options = HashMap::new();
    if input_type.get_asset().id.token_subtype() == AssetSubtype::TOKEN && recipient_token_address.is_none() {
        options.insert(
            FeeOption::TokenAccountCreation,
            BigInt::from(input_type.get_asset().id.chain.token_activation_fee().unwrap_or(0)),
        );
    }
    TransactionFee::new_gas_price_type(gas_price_type.clone(), get_gas_limit(input_type), options)
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
        TransactionInputType::Transfer(_)
        | TransactionInputType::Deposit(_)
        | TransactionInputType::TokenApprove(_, _)
        | TransactionInputType::Generic(_, _, _)
        | TransactionInputType::Perpetual(_, _) => BigInt::from(100_000),
        TransactionInputType::Swap(_, _, _) => BigInt::from(420_000),
        TransactionInputType::Stake(_, _) => BigInt::from(100_000),
    }
}

fn get_multiple_of(input_type: &TransactionInputType) -> i64 {
    match input_type {
        TransactionInputType::Transfer(asset)
        | TransactionInputType::Deposit(asset)
        | TransactionInputType::TokenApprove(asset, _)
        | TransactionInputType::Generic(asset, _, _)
        | TransactionInputType::Perpetual(asset, _) => match &asset.id.token_subtype() {
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

    let priority_fee_base = if fees.is_empty() {
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
            let priority_fee = match priority {
                FeePriority::Slow => &priority_fee_base / 2,
                FeePriority::Normal => priority_fee_base.clone(),
                FeePriority::Fast => &priority_fee_base * 3,
            };

            FeeRate::new(
                *priority,
                GasPriceType::solana(
                    static_base_fee.clone(),
                    (priority_fee.clone() * gas_limit.clone()) / BigInt::from(1_000_000),
                    priority_fee.clone(),
                ),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, AssetId, AssetType, Chain, SwapProvider};
    use primitives::swap::{SwapData, SwapQuote, SwapQuoteData, SwapProviderData};

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

        // Expected calculation:
        // gas_price = 5000
        // priority_fee = 15000
        // fee = gas_price + priority_fee = 5000 + 15000 = 20000
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
            SwapData {
                quote: SwapQuote {
                    from_value: "1000000000".to_string(),
                    to_value: "1000000".to_string(),
                    provider_data: SwapProviderData {
                        provider: SwapProvider::Jupiter,
                        name: "Jupiter".to_string(),
                        protocol_name: "jupiter".to_string(),
                    },
                    wallet_address: "test".to_string(),
                    slippage_bps: 50,
                    eta_in_seconds: None,
                },
                data: SwapQuoteData {
                    to: "test".to_string(),
                    value: "0".to_string(),
                    data: "0x".to_string(),
                    approval: None,
                    gas_limit: None,
                },
            },
        );

        let fee = calculate_transaction_fee(&input_type, &gas_price_type, None);

        // Expected calculation:
        // gas_price = 5000
        // priority_fee = 30000
        // fee = gas_price + priority_fee = 5000 + 30000 = 35000
        assert_eq!(fee.fee, BigInt::from(35_000u64));
        assert_eq!(fee.gas_limit, BigInt::from(420_000u64));
    }

    #[test]
    fn test_calculate_transaction_fee_zero_priority_fee() {
        // Test with zero priority fee
        let gas_price_type = GasPriceType::eip1559(BigInt::from(5000u64), BigInt::from(0u64));
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

        // Expected calculation:
        // gas_price = 5000
        // priority_fee = 0
        // fee = gas_price + priority_fee = 5000 + 0 = 5000
        assert_eq!(fee.fee, BigInt::from(5000u64));
        assert_eq!(fee.gas_price_type.priority_fee(), BigInt::from(0u64));
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
        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(1_250));
        assert_eq!(rates[0].gas_price_type.unit_price(), BigInt::from(12_500));

        assert_eq!(rates[1].priority, FeePriority::Normal);
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(2_500));
        assert_eq!(rates[1].gas_price_type.unit_price(), BigInt::from(25_000));

        assert_eq!(rates[2].priority, FeePriority::Fast);
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(7_500));
        assert_eq!(rates[2].gas_price_type.unit_price(), BigInt::from(75_000));
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

        // Should return 3 fee rates even with empty fees
        assert_eq!(rates.len(), 3);

        // When fees are empty, priority_fee_base = multiple_of = 25_000 for native transfers
        // Slow: 25_000 / 2 = 12_500, scaled: (12_500 * 100_000) / 1_000_000 = 1_250
        // Normal: 25_000, scaled: (25_000 * 100_000) / 1_000_000 = 2_500
        // Fast: 25_000 * 3 = 75_000, scaled: (75_000 * 100_000) / 1_000_000 = 7_500
        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(1_250u64));
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(2_500u64));
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(7_500u64));
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

        // For SPL tokens, multiple_of = 50_000
        // 80_000 rounded up to nearest 50_000 = 100_000
        // priority_fee_base = max(100_000, 50_000) = 100_000
        // Slow: 100_000 / 2 = 50_000, scaled: (50_000 * 100_000) / 1_000_000 = 5_000
        // Normal: 100_000, scaled: (100_000 * 100_000) / 1_000_000 = 10_000
        // Fast: 100_000 * 3 = 300_000, scaled: (300_000 * 100_000) / 1_000_000 = 30_000
        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(5_000u64));
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(10_000u64));
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(30_000u64));
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
            SwapData {
                quote: SwapQuote {
                    from_value: "1000000000".to_string(),
                    to_value: "1000000".to_string(),
                    provider_data: SwapProviderData {
                        provider: SwapProvider::Jupiter,
                        name: "Jupiter".to_string(),
                        protocol_name: "jupiter".to_string(),
                    },
                    wallet_address: "test".to_string(),
                    slippage_bps: 50,
                    eta_in_seconds: None,
                },
                data: SwapQuoteData {
                    to: "test".to_string(),
                    value: "0".to_string(),
                    data: "0x".to_string(),
                    approval: None,
                    gas_limit: None,
                },
            },
        );

        let rates = calculate_fee_rates(&input_type, &fees);
        assert_eq!(rates.len(), 3);

        // For swaps, multiple_of = 100_000, gas_limit = 420_000
        // 150_000 rounded up to nearest 100_000 = 200_000
        // priority_fee_base = max(200_000, 100_000) = 200_000
        // Slow: 200_000 / 2 = 100_000, scaled: (100_000 * 420_000) / 1_000_000 = 42_000
        // Normal: 200_000, scaled: (200_000 * 420_000) / 1_000_000 = 84_000
        // Fast: 200_000 * 3 = 600_000, scaled: (600_000 * 420_000) / 1_000_000 = 252_000
        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(42_000u64));
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(84_000u64));
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(252_000u64));
    }

    #[test]
    fn test_calculate_fee_rates_multiple_fees() {
        // Test with multiple prioritization fees - should use top 5 and calculate average
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

        // Top 5 fees (sorted desc): [225_000, 200_000, 175_000, 150_000, 125_000]
        // Average = (225_000 + 200_000 + 175_000 + 150_000 + 125_000) / 5 = 175_000
        // Rounded up to nearest 25_000 = 175_000
        // priority_fee_base = max(175_000, 25_000) = 175_000
        // Slow: 175_000 / 2 = 87_500, scaled: (87_500 * 100_000) / 1_000_000 = 8_750
        // Normal: 175_000, scaled: (175_000 * 100_000) / 1_000_000 = 17_500
        // Fast: 175_000 * 3 = 525_000, scaled: (525_000 * 100_000) / 1_000_000 = 52_500
        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(8_750u64));
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(17_500u64));
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(52_500u64));
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

        // priority_fee_base = 150_000 (already rounded to multiple of 25_000)
        // Slow: 150_000 / 2 = 75_000, scaled: (75_000 * 100_000) / 1_000_000 = 7_500
        // Normal: 150_000, scaled: (150_000 * 100_000) / 1_000_000 = 15_000
        // Fast: 150_000 * 3 = 450_000, scaled: (450_000 * 100_000) / 1_000_000 = 45_000

        assert_eq!(rates[0].gas_price_type.priority_fee(), BigInt::from(7_500));
        assert_eq!(rates[1].gas_price_type.priority_fee(), BigInt::from(15_000));
        assert_eq!(rates[2].gas_price_type.priority_fee(), BigInt::from(45_000));
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
