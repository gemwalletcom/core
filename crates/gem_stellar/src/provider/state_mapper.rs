use crate::models::fee::StellarFees;
use primitives::{FeePriority, FeePriorityValue};

pub fn map_fee_stats_to_priorities(fees: &StellarFees) -> Vec<FeePriorityValue> {
    let min_fee = std::cmp::max(
        fees.fee_charged.min.parse::<u64>().unwrap_or(100),
        fees.last_ledger_base_fee.parse::<u64>().unwrap_or(100),
    );

    let fast_fee = fees.fee_charged.p95.parse::<u64>().unwrap_or(min_fee) * 2;

    vec![
        FeePriorityValue::new(FeePriority::Slow, min_fee.to_string()),
        FeePriorityValue::new(FeePriority::Normal, min_fee.to_string()),
        FeePriorityValue::new(FeePriority::Fast, fast_fee.to_string()),
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
        assert_eq!(result[0].value, "150"); // max(100, 150)
        assert_eq!(result[2].value, "1000"); // 500 * 2
    }
}
