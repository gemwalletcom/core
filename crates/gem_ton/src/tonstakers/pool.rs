use std::error::Error;

use num_bigint::BigUint;

use gem_client::{Client, ClientExt};

use crate::models::RunGetMethodResult;
use crate::rpc::client::TonClient;

// Stack offsets in the Tonstakers pool `get_pool_full_data` get-method tuple.
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

    const UNUSED_SIGNED_NUM_INDEX: usize = 10;

    fn stack_num(value: &str) -> Value {
        json!({ "type": "num", "value": value })
    }

    #[test]
    fn test_pool_full_data_from_stack() {
        let mut stack = vec![stack_num("0x0"); POOL_FULL_DATA_SUPPLY_INDEX + 1];
        stack[POOL_FULL_DATA_TOTAL_BALANCE_INDEX] = stack_num("0xc387ceec28e89a");
        stack[UNUSED_SIGNED_NUM_INDEX] = stack_num("-0x1");
        stack[POOL_FULL_DATA_SUPPLY_INDEX] = stack_num("0xb2c9f05e933a60");

        let value = json!({ "stack": stack });
        let result: RunGetMethodResult = serde_json::from_value(value).unwrap();
        let data = PoolFullData::from_stack(&result).unwrap();

        assert_eq!(data.total_balance, BigUint::from(55_036_943_253_694_618_u64));
        assert_eq!(data.supply, BigUint::from(50_324_580_070_537_824_u64));
    }
}
