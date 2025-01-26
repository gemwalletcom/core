use super::client::Pool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CetusPool {
    pool_address: String,
    coin_type_a: String,
    coin_type_b: String,
    coin_amount_a: String,
    coin_amount_b: String,
    current_sqrt_price: String,
    current_tick_index: i32,
    fee_rate: String,
    is_pause: bool,
    liquidity: String,
    tick_spacing: i32,
    name: String,
}

impl From<Pool> for CetusPool {
    fn from(pool: Pool) -> Self {
        Self {
            pool_address: pool.address,
            coin_type_a: pool.coin_a_address,
            coin_type_b: pool.coin_b_address,
            coin_amount_a: pool.object.current_sqrt_price.clone(),
            coin_amount_b: pool.object.liquidity.clone(),
            current_sqrt_price: pool.object.current_sqrt_price.clone(),
            current_tick_index: pool.object.tick_spacing,
            fee_rate: pool.fee,
            is_pause: pool.object.is_pause,
            liquidity: pool.object.liquidity.clone(),
            tick_spacing: pool.object.tick_spacing,
            name: pool.name,
        }
    }
}
