use std::cmp::max;

pub const GAS_BUDGET_MULTIPLIER: u64 = 120;

pub fn calculate_gas_budget(computation_cost: u64, storage_cost: u64, storage_rebate: u64) -> u64 {
    max(computation_cost, computation_cost.saturating_add(storage_cost).saturating_sub(storage_rebate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_budget() {
        let fee = calculate_gas_budget(611_000, 2_424_400, 2_400_156);
        assert_eq!(fee, 635_244);
        assert_eq!(fee * GAS_BUDGET_MULTIPLIER / 100, 762_292);
    }
}
