use crate::models::fee::StellarFees;
use primitives::{FeePriority, FeeRate};

#[cfg(test)]
use {primitives::GasPriceType, num_bigint::BigInt};

pub fn map_fee_stats_to_priorities(fees: &StellarFees) -> Vec<FeeRate> {
    let min_fee = std::cmp::max(
        fees.fee_charged.min.parse::<u64>().unwrap_or(100),
        fees.last_ledger_base_fee.parse::<u64>().unwrap_or(100),
    );

    let fast_fee = fees.fee_charged.p95.parse::<u64>().unwrap_or(min_fee) * 2;

    vec![
        FeeRate::regular(FeePriority::Slow, min_fee),
        FeeRate::regular(FeePriority::Normal, min_fee),
        FeeRate::regular(FeePriority::Fast, fast_fee),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::fee::{StellarFeeCharged, StellarFees};

    #[test]
    fn test_map_fee_stats_to_priorities() {
        let fees = StellarFees {
            fee_charged: StellarFeeCharged {
                min: "100".to_string(),
                p95: "500".to_string(),
            },
            last_ledger_base_fee: "150".to_string(),
        };

        let result = map_fee_stats_to_priorities(&fees);
        assert_eq!(result.len(), 3);
        match &result[0].gas_price_type {
            GasPriceType::Regular { gas_price } => assert_eq!(gas_price, &BigInt::from(150)), // max(100, 150)
            _ => panic!("Expected Regular gas price"),
        }
        match &result[2].gas_price_type {
            GasPriceType::Regular { gas_price } => assert_eq!(gas_price, &BigInt::from(1000)), // 500 * 2
            _ => panic!("Expected Regular gas price"),
        }
    }
}
