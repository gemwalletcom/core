use crate::models::fee::StellarFees;
use primitives::{FeePriority, FeeRate, GasPriceType};

#[cfg(test)]
use num_bigint::BigInt;

pub fn map_fee_stats_to_priorities(fees: &StellarFees) -> Vec<FeeRate> {
    let min_fee = std::cmp::max(fees.fee_charged.min, fees.last_ledger_base_fee);

    let fast_fee = fees.fee_charged.p95.max(min_fee) * 2;

    vec![
        FeeRate::new(FeePriority::Slow, GasPriceType::regular(min_fee)),
        FeeRate::new(FeePriority::Normal, GasPriceType::regular(min_fee)),
        FeeRate::new(FeePriority::Fast, GasPriceType::regular(fast_fee)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::fee::{StellarFeeCharged, StellarFees};

    #[test]
    fn test_map_fee_stats_to_priorities() {
        let fees = StellarFees {
            fee_charged: StellarFeeCharged { min: 100, p95: 500 },
            last_ledger_base_fee: 150,
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
