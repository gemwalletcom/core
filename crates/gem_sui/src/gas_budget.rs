use std::cmp::max;

/// Returns (fee, gas_limit) where gas_limit includes a 20% buffer.
pub fn calculate_gas_budget(computation_cost: u64, storage_cost: u64, storage_rebate: u64) -> (u64, u64) {
    let fee = max(computation_cost, computation_cost + storage_cost - storage_rebate);
    (fee, fee * 120 / 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_budget() {
        let (fee, gas_limit) = calculate_gas_budget(611_000, 2_424_400, 2_400_156);
        assert_eq!(fee, 635_244);
        assert_eq!(gas_limit, 762_292);
    }
}
