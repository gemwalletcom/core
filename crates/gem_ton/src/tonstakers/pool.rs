use std::error::Error;

use num_bigint::BigUint;

use gem_client::{Client, ClientExt};

use crate::models::RunGetMethodResult;
use crate::rpc::client::TonClient;

// Stack offsets in the Tonstakers pool `get_pool_full_data` get-method tuple.
// Layout source: ton-blockchain/liquid-staking-contract `compose_pool_full_data_internal`.
const POOL_FULL_DATA_TOTAL_BALANCE_INDEX: usize = 2;
const POOL_FULL_DATA_SUPPLY_INDEX: usize = 13;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolFullData {
    pub total_balance: BigUint,
    pub supply: BigUint,
}

impl PoolFullData {
    pub fn from_stack(result: &RunGetMethodResult) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            total_balance: result.get_num(POOL_FULL_DATA_TOTAL_BALANCE_INDEX)?,
            supply: result.get_num(POOL_FULL_DATA_SUPPLY_INDEX)?,
        })
    }
}

pub async fn get_pool_full_data<C: Client>(client: &TonClient<C>, address: &str) -> Result<PoolFullData, Box<dyn Error + Send + Sync>> {
    let result: RunGetMethodResult = client
        .client
        .post(
            "/api/v3/runGetMethod",
            &serde_json::json!({
                "address": address,
                "method": "get_pool_full_data",
                "stack": [],
            }),
        )
        .await?;
    PoolFullData::from_stack(&result)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    const MAX_LOAN_PER_VALIDATOR_INDEX: usize = 10;

    fn num_stack_item(value: i64) -> Value {
        let hex = if value < 0 { format!("-0x{:x}", value.unsigned_abs()) } else { format!("0x{value:x}") };
        json!({ "type": "num", "value": hex })
    }

    #[test]
    fn test_pool_full_data_from_stack() {
        let mut stack = vec![num_stack_item(0); POOL_FULL_DATA_SUPPLY_INDEX + 1];
        stack[POOL_FULL_DATA_TOTAL_BALANCE_INDEX] = num_stack_item(55_036_943_253_694_618);
        stack[MAX_LOAN_PER_VALIDATOR_INDEX] = num_stack_item(-1);
        stack[POOL_FULL_DATA_SUPPLY_INDEX] = num_stack_item(50_324_580_070_537_824);

        let value = json!({ "stack": stack });
        let result: RunGetMethodResult = serde_json::from_value(value).unwrap();
        let data = PoolFullData::from_stack(&result).unwrap();

        assert_eq!(data.total_balance, BigUint::from(55_036_943_253_694_618_u64));
        assert_eq!(data.supply, BigUint::from(50_324_580_070_537_824_u64));
    }
}
