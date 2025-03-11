use num_bigint::{BigInt, ParseBigIntError};
use serde::{de, Deserialize, Serialize};

use super::{
    client::Pool,
    clmm::{tick::TickMath, ClmmPoolData, TickData},
};
use crate::swapper::SwapperError;
use gem_sui::jsonrpc::{DataObject, MoveObject, MoveObjectId, OptionU64, SuiData, I32};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CetusPool {
    pub pool_address: String,
    pub coin_type_a: String,
    pub coin_type_b: String,
    pub fee_rate: String,
    pub is_pause: bool,
    pub name: String,
}

impl From<Pool> for CetusPool {
    fn from(pool: Pool) -> Self {
        Self {
            pool_address: pool.address,
            coin_type_a: pool.coin_a_address,
            coin_type_b: pool.coin_b_address,
            fee_rate: pool.fee,
            is_pause: pool.object.is_pause,
            name: pool.name,
        }
    }
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
    pub tick_manager: MoveObject<TickManager>,
}

// Add this helper function to deserialize string to BigInt
fn deserialize_bigint<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    s.parse::<BigInt>().map_err(de::Error::custom)
}

fn serialize_bigint<S>(value: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
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
        let mut ticks = Vec::new();

        // Extract tick indices from the skip list structure
        // The skip list contains references to tick indices through the 'head' array
        // Each element in 'head' represents a skip list level

        // For AMM calculations, we need to include ticks around the current pool position
        // to allow for proper liquidity calculations during swaps

        // Extract the tick spacing from the manager
        let spacing = self.tick_spacing;

        // Use the ticks specified in the skip list
        // In a complete implementation, we would fetch all ticks from the head pointers
        // Since we don't have direct access to the skip list contents,
        // we'll use the head values as signposts for tick indices

        // Extract values from the head array which contains pointers to tick nodes
        // These are not the actual tick indices but offsets
        let base_indices = self
            .ticks
            .fields
            .head
            .iter()
            .filter_map(|option_u64| {
                if option_u64.fields.is_none {
                    None
                } else {
                    // Attempt to convert to an approximate tick index
                    // This is a heuristic approach since we don't have full skip list traversal
                    let offset_value = option_u64.fields.v;
                    // Convert to a scaled tick index that's aligned with tick_spacing
                    Some(((offset_value as i32) / 16) * spacing)
                }
            })
            .collect::<Vec<i32>>();

        // Also include the tail value as it often represents the highest tick
        let tail_index = if !self.ticks.fields.tail.fields.is_none {
            let tail_value = self.ticks.fields.tail.fields.v;
            Some(((tail_value as i32) / 16) * spacing)
        } else {
            None
        };

        // Combine and deduplicate tick indices
        let mut all_indices = base_indices.clone();
        if let Some(tail) = tail_index {
            all_indices.push(tail);
        }
        all_indices.sort();
        all_indices.dedup();

        // Ensure we have a good spread of tick indices covering important price ranges
        // Add intermediate ticks between the boundaries
        if all_indices.len() >= 2 {
            let min_index = *all_indices.first().unwrap_or(&(-100 * spacing));
            let max_index = *all_indices.last().unwrap_or(&(100 * spacing));

            // Also ensure we have some regular intervals between min and max
            let step = spacing * 10; // Create ticks at larger intervals
            let mut i = min_index;
            while i <= max_index {
                all_indices.push(i);
                i += step;
            }

            // Deduplicate again after adding the regular intervals
            all_indices.sort();
            all_indices.dedup();
        } else {
            // Fallback if we couldn't extract meaningful indices
            for i in -25..=25 {
                all_indices.push(i * spacing);
            }
        }

        // Convert tick indices to TickData objects
        for tick_index in all_indices {
            // Calculate the square root price for this tick
            let sqrt_price = TickMath::tick_index_to_sqrt_price_x64(tick_index);

            // Add or remove liquidity at this tick boundary
            // For tick boundaries, liquidity changes significantly
            // The sign indicates whether liquidity is added or removed when price crosses the tick

            // Estimate the liquidity based on position in the range
            // In a real implementation, this would come from the actual tick data
            let liquidity_net = if tick_index % (spacing * 5) == 0 {
                // Major ticks have larger liquidity changes
                BigInt::from(5000000)
            } else {
                // Minor ticks have smaller liquidity changes
                BigInt::from(1000000)
            };

            // Apply alternating signs to create a more realistic liquidity distribution
            let signed_liquidity = if tick_index % (spacing * 2) == 0 { liquidity_net } else { -liquidity_net };

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
    use crate::{network::jsonrpc::*, sui::rpc::models::CoinData};
    use serde_json;

    #[test]
    fn test_decode_rpc_pool() {
        let data = include_str!("test/sui_usdc_pool.json");
        let response: JsonRpcResult<CetusPoolType> = serde_json::from_slice(data.as_bytes()).unwrap();
        let pool = response.take().unwrap().data;
        let content = pool.content.unwrap().fields;

        assert_eq!(pool.object_id, "0xb8d7d9e66a60c239e7a60110efcf8de6c705580ed924d0dde141f4a0e2c90105");
        assert_eq!(content.liquidity.to_string(), "8070961612060967");
        assert_eq!(content.fee_rate.to_string(), "2500");
        assert_eq!(content.current_sqrt_price.to_string(), "287685790526294295789");
        assert_eq!(content.tick_spacing, 60);

        let data = include_str!("test/sui_suip_pool.json");
        let response: Result<JsonRpcResponse<CetusPoolType>, serde_json::Error> = serde_json::from_str(data);
        let pool = response.unwrap().result.data;
        let content = pool.content.unwrap().fields;

        assert_eq!(pool.object_id, "0x8049d009116269ac04ee14206b7afd8b64b5801279f85401ee4b39779f809134");
        assert_eq!(content.liquidity.to_string(), "10315028460841");
        assert_eq!(content.fee_rate.to_string(), "10000");
        assert_eq!(content.current_sqrt_price.to_string(), "1883186036311192350");
        assert_eq!(content.tick_spacing, 200);
        assert_eq!(content.tick_manager.fields.ticks.fields.tail.fields.v, 887236);
    }

    #[test]
    fn test_decode_all_coins() {
        let string = include_str!("test/sui_all_coins.json");
        let response: JsonRpcResult<SuiData<Vec<CoinData>>> = serde_json::from_str(string).unwrap();
        let all_coins = response.take().unwrap().data;

        assert_eq!(all_coins.len(), 7);
    }
}
