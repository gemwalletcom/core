use num_bigint::{BigInt, ParseBigIntError};
use serde::{Deserialize, Serialize};
use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber},
    digests::ObjectDigest,
};

use crate::swapper::SwapperError;
use gem_sui::jsonrpc::{DataObject, MoveObject, MoveObjectId, OptionU64, SuiData, I32};
use serde_serializers::{deserialize_bigint_from_str as deserialize_bigint, serialize_bigint};

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct CalculatedSwapResult {
    #[serde(deserialize_with = "deserialize_bigint")]
    pub amount_out: BigInt,
    #[serde(deserialize_with = "deserialize_bigint")]
    pub fee_amount: BigInt,
    pub is_exceed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePoolData {
    pub object_id: ObjectID,
    pub version: u64,
    pub digest: ObjectDigest,
    pub coin_a: String,
    pub coin_b: String,
    pub initial_shared_version: u64,
}

pub type CetusPoolType = SuiData<DataObject<MoveObject<CetusPoolObject>>>;

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
}

impl From<ParseBigIntError> for SwapperError {
    fn from(err: ParseBigIntError) -> Self {
        Self::ComputeQuoteError { msg: err.to_string() }
    }
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

#[derive(Debug, Clone)]
pub struct SwapParams {
    pub pool_object_shared: SharedObject,
    pub a2b: bool,
    pub by_amount_in: bool,
    pub amount: BigInt,
    pub amount_limit: BigInt,
    pub coin_type_a: String,
    pub coin_type_b: String,
    pub swap_partner: Option<ObjectRef>,
}

#[derive(Debug, Clone)]
pub struct CetusConfig {
    pub global_config: SharedObject,
    pub clmm_pool: ObjectID,
    pub router: ObjectID,
}

#[derive(Debug, Clone)]
pub struct SharedObject {
    pub id: ObjectID,
    pub shared_version: u64,
}

impl SharedObject {
    pub fn initial_shared_version(&self) -> SequenceNumber {
        SequenceNumber::from_u64(self.shared_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        network::jsonrpc::*,
        sui::rpc::{
            models::{InspectEvent, InspectResult},
            CoinAsset,
        },
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

        let event = result.events.as_array().unwrap().first().unwrap();
        let event_data: InspectEvent<SuiData<CalculatedSwapResult>> = serde_json::from_value(event.clone()).unwrap();

        assert!(result.error.is_none());
        assert!(result.effects.total_gas_cost() > 0);
        assert_eq!(event_data.parsed_json.data.amount_out.to_string(), "1168986");
    }
}
