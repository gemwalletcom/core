#[derive(Debug, Clone, Default)]
pub struct FeeStats {
    pub median: i64,
    pub p75: i64,
    pub p90: i64,
    pub avg: i64,
    pub count: usize,
}

fn percentile(sorted_values: &[i64], p: usize) -> i64 {
    if sorted_values.is_empty() {
        return 0;
    }
    let idx = (p * sorted_values.len() / 100).min(sorted_values.len() - 1);
    sorted_values[idx]
}

pub fn calculate_fee_stats(fees: &[i64]) -> FeeStats {
    if fees.is_empty() {
        return FeeStats::default();
    }

    let mut values = fees.to_vec();
    values.sort();

    let count = values.len();
    let sum: i64 = values.iter().sum();

    FeeStats {
        median: percentile(&values, 50),
        p75: percentile(&values, 75),
        p90: percentile(&values, 90),
        avg: sum / count as i64,
        count,
    }
}
