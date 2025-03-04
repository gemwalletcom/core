use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::client::Pool;
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
    pub current_sqrt_price: String,
    pub current_tick_index: MoveObject<I32>,
    pub fee_rate: String,
    pub id: MoveObjectId,
    pub is_pause: bool,
    pub tick_spacing: i32,
    pub liquidity: String,
    pub tick_manager: MoveObject<TickManager>,
}

impl CetusPoolObject {
    pub fn fee_rate(&self) -> Result<BigInt, SwapperError> {
        BigInt::from_str(&self.fee_rate).map_err(|_| SwapperError::ComputeQuoteError {
            msg: "Invalid fee_rate".into(),
        })
    }

    pub fn current_sqrt_price(&self) -> Result<BigInt, SwapperError> {
        BigInt::from_str(&self.current_sqrt_price).map_err(|_| SwapperError::ComputeQuoteError {
            msg: "Invalid current_sqrt_price".into(),
        })
    }

    pub fn liquidity(&self) -> Result<BigInt, SwapperError> {
        BigInt::from_str(&self.liquidity).map_err(|_| SwapperError::ComputeQuoteError {
            msg: "Invalid liquidity".into(),
        })
    }

    pub fn current_tick_index(&self) -> i32 {
        self.current_tick_index.fields.bits
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::jsonrpc::*;

    #[test]
    fn test_decode_rpc_pool() {
        let data = include_str!("test/pool_object.json");
        let response: JsonRpcResult<CetusPoolType> = serde_json::from_slice(data.as_bytes()).unwrap();
        let pool = response.take().unwrap().data;
        let content = pool.content.unwrap().fields;

        assert_eq!(pool.object_id, "0xb8d7d9e66a60c239e7a60110efcf8de6c705580ed924d0dde141f4a0e2c90105");
        assert_eq!(content.liquidity, "8070961612060967");
        assert_eq!(content.fee_rate, "2500");
        assert_eq!(content.current_sqrt_price, "287685790526294295789");
        assert_eq!(content.tick_spacing, 60);
    }
}
