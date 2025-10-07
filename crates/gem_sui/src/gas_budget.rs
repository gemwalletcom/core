use crate::models::InspectGasUsed;
use std::cmp::max;

pub struct GasBudgetCalculator {}

impl GasBudgetCalculator {
    pub fn gas_budget(gas_used: &InspectGasUsed) -> u64 {
        let computation_budget = gas_used.computation_cost;
        let budget = max(computation_budget, computation_budget + gas_used.storage_cost - gas_used.storage_rebate);
        budget * 120 / 100
    }
}
