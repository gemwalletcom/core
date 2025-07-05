use num_bigint::{BigInt, BigUint};
use std::collections::HashMap;

use primitives::{AssetId, TransactionSwapMetadata};

/// Address -> Vec<BalanceDiff>
pub type BalanceDiffMap = HashMap<String, Vec<BalanceDiff>>;

#[derive(Debug)]
pub struct BalanceDiff {
    pub asset_id: AssetId,
    pub from_value: Option<BigInt>,
    pub to_value: Option<BigInt>,
    pub diff: BigInt,
}

pub struct SwapMapper;

impl SwapMapper {
    /// Maps a set of balance changes to swap metadata if they represent a swap transaction
    pub fn map_swap(balance_diffs: &[BalanceDiff], fee: &BigUint, native_asset_id: &AssetId, provider: Option<String>) -> Option<TransactionSwapMetadata> {
        let non_zero_diffs: Vec<&BalanceDiff> = balance_diffs.iter().filter(|diff| diff.diff != BigInt::from(0)).collect();

        if non_zero_diffs.len() != 2 {
            return None;
        }

        let first = non_zero_diffs.first()?;
        let second = non_zero_diffs.last()?;

        // One should be positive (received), one negative (sent)
        if (first.diff > BigInt::from(0)) == (second.diff > BigInt::from(0)) {
            return None;
        }

        let (sent_diff, received_diff) = if first.diff < BigInt::from(0) { (first, second) } else { (second, first) };

        let from_value = Self::calculate_actual_value(&sent_diff.diff, &sent_diff.asset_id, fee, native_asset_id);
        let to_value = Self::calculate_actual_value(&received_diff.diff, &received_diff.asset_id, fee, native_asset_id);

        // Ignore Mint txs
        if from_value == BigUint::from(0u8) {
            return None;
        }

        Some(TransactionSwapMetadata {
            from_asset: sent_diff.asset_id.clone(),
            from_value: from_value.to_string(),
            to_asset: received_diff.asset_id.clone(),
            to_value: to_value.to_string(),
            provider,
        })
    }

    /// Calculates the actual value of a balance change, accounting for transaction fees
    /// For native tokens, we need to subtract the fee from the amount since the balance change includes both the swap amount and the fee payment.
    fn calculate_actual_value(amount: &BigInt, asset_id: &AssetId, fee: &BigUint, native_asset_id: &AssetId) -> BigUint {
        let magnitude = amount.magnitude();
        if asset_id == native_asset_id && magnitude >= fee {
            magnitude - fee
        } else {
            magnitude.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_detect_simple_swap() {
        let native_asset = AssetId::from_chain(Chain::Ethereum);
        let token_asset = AssetId::from_token(Chain::Ethereum, "0x123");
        let fee = BigUint::from(1000u32);

        let balance_diffs = vec![
            BalanceDiff {
                asset_id: native_asset.clone(),
                from_value: Some(BigInt::from(5000)),
                to_value: Some(BigInt::from(0)),
                diff: BigInt::from(-5000),
            },
            BalanceDiff {
                asset_id: token_asset.clone(),
                from_value: None,
                to_value: None,
                diff: BigInt::from(100),
            },
        ];

        let swap = SwapMapper::map_swap(&balance_diffs, &fee, &native_asset, Some("Uniswap".to_string())).unwrap();

        assert_eq!(swap.from_asset, native_asset);
        assert_eq!(swap.from_value, "4000");
        assert_eq!(swap.to_asset, token_asset);
        assert_eq!(swap.to_value, "100");
        assert_eq!(swap.provider, Some("Uniswap".to_string()));
    }

    #[test]
    fn test_detect_token_to_token_swap() {
        let native_asset = AssetId::from_chain(Chain::Ethereum);
        let token_a = AssetId::from_token(Chain::Ethereum, "0x123");
        let token_b = AssetId::from_token(Chain::Ethereum, "0x456");
        let fee = BigUint::from(1000u32);

        let balance_diffs = vec![
            BalanceDiff {
                asset_id: token_a.clone(),
                from_value: None,
                to_value: None,
                diff: BigInt::from(-200),
            },
            BalanceDiff {
                asset_id: token_b.clone(),
                from_value: None,
                to_value: None,
                diff: BigInt::from(150),
            },
        ];

        let swap = SwapMapper::map_swap(&balance_diffs, &fee, &native_asset, Some("Uniswap".to_string())).unwrap();

        assert_eq!(swap.from_asset, token_a);
        assert_eq!(swap.from_value, "200");
        assert_eq!(swap.to_asset, token_b);
        assert_eq!(swap.to_value, "150");
    }

    #[test]
    fn test_not_a_swap_same_direction() {
        let native_asset = AssetId::from_chain(Chain::Ethereum);
        let token_asset = AssetId::from_token(Chain::Ethereum, "0x123");
        let fee = BigUint::from(1000u32);

        let balance_diffs = vec![
            BalanceDiff {
                asset_id: native_asset.clone(),
                from_value: None,
                to_value: None,
                diff: BigInt::from(5000),
            },
            BalanceDiff {
                asset_id: token_asset,
                from_value: None,
                to_value: None,
                diff: BigInt::from(100),
            },
        ];

        let swap = SwapMapper::map_swap(&balance_diffs, &fee, &native_asset, Some("Uniswap".to_string()));

        assert!(swap.is_none());
    }

    #[test]
    fn test_not_a_swap_wrong_count() {
        let native_asset = AssetId::from_chain(Chain::Ethereum);
        let fee = BigUint::from(1000u32);

        let balance_diffs = vec![BalanceDiff {
            asset_id: native_asset.clone(),
            from_value: None,
            to_value: None,
            diff: BigInt::from(-5000),
        }];

        let swap = SwapMapper::map_swap(&balance_diffs, &fee, &native_asset, Some("Uniswap".to_string()));

        assert!(swap.is_none());
    }

    #[test]
    fn test_swap_detection_with_zero_diffs() {
        let native_asset = AssetId::from_chain(Chain::Ethereum);
        let token_asset = AssetId::from_token(Chain::Ethereum, "0x123");
        let fee = BigUint::from(1000u32);

        let balance_diffs = vec![
            BalanceDiff {
                asset_id: native_asset.clone(),
                from_value: None,
                to_value: None,
                diff: BigInt::from(-5000),
            },
            BalanceDiff {
                asset_id: token_asset.clone(),
                from_value: None,
                to_value: None,
                diff: BigInt::from(100),
            },
            BalanceDiff {
                asset_id: AssetId::from_token(Chain::Ethereum, "0x789"),
                from_value: None,
                to_value: None,
                diff: BigInt::from(0),
            },
        ];

        let swap = SwapMapper::map_swap(&balance_diffs, &fee, &native_asset, Some("Uniswap".to_string())).unwrap();

        assert_eq!(swap.from_asset, native_asset);
        assert_eq!(swap.to_asset, token_asset);
    }
}
