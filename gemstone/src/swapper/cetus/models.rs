use num_bigint::{BigInt, ParseBigIntError};
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_bigint_from_str as deserialize_bigint, serialize_bigint};
use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber},
    digests::ObjectDigest,
};

use super::clmm::{tick::TickMath, ClmmPoolData, TickData};
use crate::swapper::SwapperError;
use gem_sui::jsonrpc::{DataObject, MoveObject, MoveObjectId, OptionU64, SuiData, I32};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePoolData {
    pub object_id: ObjectID,
    pub version: u64,
    pub digest: ObjectDigest,
    pub coin_a: String,
    pub coin_b: String,
    pub initial_shared_version: u64,
}

impl RoutePoolData {
    #[allow(unused)]
    pub fn obj_ref(&self) -> ObjectRef {
        (self.object_id, SequenceNumber::from_u64(self.version), self.digest)
    }
}

pub type CetusPoolType = SuiData<DataObject<MoveObject<CetusPoolObject>>>;

pub fn get_pool_object(data: &CetusPoolType) -> Option<CetusPoolObject> {
    data.data.content.clone().map(|content| content.fields)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CetusPoolObject {
    pub coin_a: String,
    pub coin_b: String,
    #[serde(deserialize_with = "deserialize_bigint", serialize_with = "serialize_bigint")]
    pub current_sqrt_price: BigInt,
    pub current_tick_index: MoveObject<I32>,
    #[serde(deserialize_with = "deserialize_bigint", serialize_with = "serialize_bigint")]
    pub fee_growth_global_a: BigInt,
    #[serde(deserialize_with = "deserialize_bigint", serialize_with = "serialize_bigint")]
    pub fee_growth_global_b: BigInt,
    #[serde(deserialize_with = "deserialize_bigint", serialize_with = "serialize_bigint")]
    pub fee_protocol_coin_a: BigInt,
    #[serde(deserialize_with = "deserialize_bigint", serialize_with = "serialize_bigint")]
    pub fee_protocol_coin_b: BigInt,
    #[serde(deserialize_with = "deserialize_bigint", serialize_with = "serialize_bigint")]
    pub fee_rate: BigInt,
    pub id: MoveObjectId,
    pub is_pause: bool,
    pub tick_spacing: i32,
    #[serde(deserialize_with = "deserialize_bigint", serialize_with = "serialize_bigint")]
    pub liquidity: BigInt,
    pub tick_manager: MoveObject<TickManager>,
}

impl TryInto<ClmmPoolData> for CetusPoolObject {
    type Error = SwapperError;

    fn try_into(self) -> Result<ClmmPoolData, Self::Error> {
        Ok(ClmmPoolData {
            liquidity: self.liquidity,
            current_sqrt_price: self.current_sqrt_price,
            current_tick_index: self.current_tick_index.fields.bits,
            fee_rate: self.fee_rate,
        })
    }
}

impl From<ParseBigIntError> for SwapperError {
    fn from(err: ParseBigIntError) -> Self {
        Self::ComputeQuoteError { msg: err.to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickManager {
    pub tick_spacing: i32,
    pub ticks: MoveObject<Tick>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub head: Vec<MoveObject<OptionU64>>,
    pub id: MoveObjectId,
    pub level: String,
    pub list_p: String,
    pub max_level: String,
    pub size: String,
    pub tail: MoveObject<OptionU64>,
}

impl TickManager {
    pub fn to_ticks(&self) -> Vec<TickData> {
        const TICK_SCALING_FACTOR: i32 = 16;
        const DEFAULT_MIN_TICKS: i32 = -25;
        const DEFAULT_MAX_TICKS: i32 = 25;
        const DEFAULT_RANGE_MULTIPLIER: i32 = 100;
        const MAJOR_TICK_MULTIPLIER: i32 = 5;
        const TICK_ALTERNATION_MULTIPLIER: i32 = 2;
        const STEP_MULTIPLIER: i32 = 10;
        const MAJOR_TICK_LIQUIDITY: i64 = 5000000;
        const MINOR_TICK_LIQUIDITY: i64 = 1000000;

        let mut ticks = Vec::new();
        let spacing = self.tick_spacing;

        let base_indices = self
            .ticks
            .fields
            .head
            .iter()
            .filter_map(|option_u64| {
                if !option_u64.fields.is_none {
                    let offset_value = option_u64.fields.v;
                    Some(((offset_value as i32) / TICK_SCALING_FACTOR) * spacing)
                } else {
                    None
                }
            })
            .collect::<Vec<i32>>();

        let tail_index = if !self.ticks.fields.tail.fields.is_none {
            let tail_value = self.ticks.fields.tail.fields.v;
            Some(((tail_value as i32) / TICK_SCALING_FACTOR) * spacing)
        } else {
            None
        };

        let mut all_indices = base_indices;
        if let Some(tail) = tail_index {
            all_indices.push(tail);
        }
        all_indices.sort();
        all_indices.dedup();

        if all_indices.len() >= 2 {
            let min_index = *all_indices.first().unwrap_or(&(-DEFAULT_RANGE_MULTIPLIER * spacing));
            let max_index = *all_indices.last().unwrap_or(&(DEFAULT_RANGE_MULTIPLIER * spacing));

            let step = spacing * STEP_MULTIPLIER;
            for i in (min_index..=max_index).step_by(step as usize) {
                all_indices.push(i);
            }

            all_indices.sort();
            all_indices.dedup();
        } else {
            for i in DEFAULT_MIN_TICKS..=DEFAULT_MAX_TICKS {
                all_indices.push(i * spacing);
            }
        }

        for tick_index in all_indices {
            let sqrt_price = TickMath::tick_index_to_sqrt_price_x64(tick_index);

            let liquidity_net = if tick_index % (spacing * MAJOR_TICK_MULTIPLIER) == 0 {
                BigInt::from(MAJOR_TICK_LIQUIDITY)
            } else {
                BigInt::from(MINOR_TICK_LIQUIDITY)
            };

            let signed_liquidity = if tick_index % (spacing * TICK_ALTERNATION_MULTIPLIER) == 0 {
                liquidity_net
            } else {
                -liquidity_net
            };

            ticks.push(TickData {
                index: tick_index,
                sqrt_price,
                liquidity_net: signed_liquidity,
            });
        }

        ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        network::jsonrpc::*,
        sui::rpc::{models::InspectResult, CoinAsset},
    };
    use serde_json;

    #[test]
    fn test_decode_rpc_pool() {
        let data = include_str!("test/sui_usdc_pool.json");
        let response: JsonRpcResponse<CetusPoolType> = serde_json::from_slice(data.as_bytes()).unwrap();
        let pool = response.result.data;
        let content = pool.content.unwrap().fields;

        assert_eq!(pool.object_id.to_hex(), "b8d7d9e66a60c239e7a60110efcf8de6c705580ed924d0dde141f4a0e2c90105");
        assert_eq!(content.liquidity.to_string(), "8070961612060967");
        assert_eq!(content.fee_rate.to_string(), "2500");
        assert_eq!(content.current_sqrt_price.to_string(), "287685790526294295789");
        assert_eq!(content.tick_spacing, 60);

        let data = include_str!("test/sui_suip_pool.json");
        let response: Result<JsonRpcResponse<CetusPoolType>, serde_json::Error> = serde_json::from_str(data);
        let pool = response.unwrap().result.data;
        let content = pool.content.unwrap().fields;

        assert_eq!(pool.object_id.to_hex(), "8049d009116269ac04ee14206b7afd8b64b5801279f85401ee4b39779f809134");
        assert_eq!(content.liquidity.to_string(), "10315028460841");
        assert_eq!(content.fee_rate.to_string(), "10000");
        assert_eq!(content.current_sqrt_price.to_string(), "1883186036311192350");
        assert_eq!(content.tick_spacing, 200);
        assert_eq!(content.tick_manager.fields.ticks.fields.tail.fields.v, 887236);
    }

    #[test]
    fn test_decode_all_coins() {
        let string = include_str!("test/sui_all_coins.json");
        let response: JsonRpcResponse<SuiData<Vec<CoinAsset>>> = serde_json::from_str(string).unwrap();
        let all_coins = response.result.data;

        assert_eq!(all_coins.len(), 7);
    }

    #[test]
    fn test_decode_dev_inspect() {
        let string = include_str!("test/sui_dev_inspect.json");
        let response: JsonRpcResponse<InspectResult> = serde_json::from_str(string).unwrap();
        let result = response.result;

        assert!(result.error.is_some());
        assert!(result.effects.total_gas_cost() > 0);
    }
}
