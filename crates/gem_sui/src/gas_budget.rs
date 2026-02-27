use crate::models::GasUsed;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::cmp::max;

pub struct GasBudgetCalculator;

impl GasBudgetCalculator {
    pub fn total_gas(gas_used: &GasUsed) -> u64 {
        let computation = &gas_used.computation_cost;
        let storage = &gas_used.storage_cost;
        let rebate = &gas_used.storage_rebate;
        let net = if computation + storage > *rebate {
            computation + storage - rebate
        } else {
            BigUint::from(0u64)
        };
        max(computation.clone(), net).to_u64().unwrap_or(0)
    }
}
