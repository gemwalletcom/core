use super::constants::*;
use super::error::{ClmmpoolsError, CoinErrorCode, MathErrorCode};
use super::swap::SwapUtils;
use super::tick::TickMath;
use super::utils::MathUtil;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use num_traits::{One, Zero};
use std::str::FromStr;

fn floor_bigdecimal(n: &BigDecimal) -> BigDecimal {
    let truncated = n.with_scale(0); // Truncate the decimal part
    if *n >= truncated {
        truncated
    } else {
        truncated - BigDecimal::one() // Round down for negative numbers
    }
}

fn ceil_bigdecimal(n: &BigDecimal) -> BigDecimal {
    let truncated = n.with_scale(0);
    if *n <= truncated {
        truncated
    } else {
        truncated + BigDecimal::one() // Round up for positive numbers
    }
}

/// Result of a swap step computation
pub struct SwapStepResult {
    pub amount_in: BigInt,
    pub amount_out: BigInt,
    pub next_sqrt_price: BigInt,
    pub fee_amount: BigInt,
}

/// Result of a swap computation
pub struct SwapResult {
    pub amount_in: BigInt,
    pub amount_out: BigInt,
    pub fee_amount: BigInt,
    pub ref_amount: BigInt,
    pub next_sqrt_price: BigInt,
    pub cross_tick_num: usize,
}

/// Coin amounts for both tokens
pub struct CoinAmounts {
    pub coin_a: BigInt,
    pub coin_b: BigInt,
}

/// Convert simple values to CoinAmounts
pub fn to_coin_amount(a: i64, b: i64) -> CoinAmounts {
    CoinAmounts {
        coin_a: BigInt::from(a),
        coin_b: BigInt::from(b),
    }
}

/// Get the amount A delta about two prices, for give amount of liquidity.
/// `delta_a = (liquidity * delta_sqrt_price) / sqrt_price_upper * sqrt_price_lower)`
///
/// # Arguments
///
/// * `sqrt_price0` - A sqrt price
/// * `sqrt_price1` - Another sqrt price
/// * `liquidity` - The amount of usable liquidity
/// * `round_up` - Whether to round the amount up or down
///
/// # Returns
///
/// * The amount A delta
pub fn get_delta_a(sqrt_price0: &BigInt, sqrt_price1: &BigInt, liquidity: &BigInt, round_up: bool) -> BigInt {
    let sqrt_price_diff = if sqrt_price0 > sqrt_price1 {
        sqrt_price0 - sqrt_price1
    } else {
        sqrt_price1 - sqrt_price0
    };

    let numerator = (liquidity * &sqrt_price_diff) << 64;
    let denominator = sqrt_price0 * sqrt_price1;
    let quotient = &numerator / &denominator;
    let remainder = numerator % denominator;

    if round_up && remainder > *ZERO {
        quotient + 1
    } else {
        quotient
    }
    // No overflow check as it's commented out in the original code
}

/// Get the amount B delta about two prices, for give amount of liquidity.
/// `delta_b = (liquidity * delta_sqrt_price)`
///
/// # Arguments
///
/// * `sqrt_price0` - A sqrt price
/// * `sqrt_price1` - Another sqrt price
/// * `liquidity` - The amount of usable liquidity
/// * `round_up` - Whether to round the amount up or down
///
/// # Returns
///
/// * The amount B delta
pub fn get_delta_b(sqrt_price0: &BigInt, sqrt_price1: &BigInt, liquidity: &BigInt, round_up: bool) -> Result<BigInt, ClmmpoolsError> {
    let sqrt_price_diff = if sqrt_price0 > sqrt_price1 {
        sqrt_price0 - sqrt_price1
    } else {
        sqrt_price1 - sqrt_price0
    };

    if liquidity == &*ZERO || sqrt_price_diff == *ZERO {
        return Ok(ZERO.clone());
    }

    let p = liquidity * &sqrt_price_diff;
    let should_round_up = round_up && (&p & &*U64_MAX) > *ZERO;
    let result = if should_round_up { &p >> (64 + 1) } else { &p >> 64 };

    if MathUtil::is_overflow(&result, 64) {
        Err(ClmmpoolsError::math_error("Result large than u64 max", MathErrorCode::IntegerDowncastOverflow))
    } else {
        Ok(result)
    }
}

/// Get the next sqrt price from give a delta of token_a.
/// `new_sqrt_price = (sqrt_price * liquidity) / (liquidity +/- amount * sqrt_price)`
///
/// # Arguments
///
/// * `sqrt_price` - The start sqrt price
/// * `liquidity` - The amount of usable liquidity
/// * `amount` - The amount of token_a
/// * `by_amount_in` - Weather to fixed input
///
/// # Returns
///
/// * The next sqrt price
pub fn get_next_sqrt_price_a_up(sqrt_price: &BigInt, liquidity: &BigInt, amount: &BigInt, by_amount_in: bool) -> Result<BigInt, ClmmpoolsError> {
    if amount == &*ZERO {
        return Ok(sqrt_price.clone());
    }

    let numerator = MathUtil::check_mul_shift_left(sqrt_price, liquidity, 64, 256)?;
    let liquidity_shl64 = liquidity << 64;
    let product = MathUtil::check_mul(sqrt_price, amount, 256)?;

    if !by_amount_in && liquidity_shl64 <= product {
        return Err(ClmmpoolsError::math_error(
            "getNextSqrtPriceAUp - Unable to divide liquidityShl64 by product",
            MathErrorCode::DivideByZero,
        ));
    }

    let next_sqrt_price = if by_amount_in {
        MathUtil::check_div_round_up_if(&numerator, &(liquidity_shl64 + product), true)?
    } else {
        MathUtil::check_div_round_up_if(&numerator, &(liquidity_shl64 - product), true)?
    };

    if next_sqrt_price < *MIN_SQRT_PRICE {
        return Err(ClmmpoolsError::coin_error(
            "getNextSqrtPriceAUp - Next sqrt price less than min sqrt price",
            CoinErrorCode::CoinAmountMinSubceeded,
        ));
    }

    if next_sqrt_price > *MAX_SQRT_PRICE {
        return Err(ClmmpoolsError::coin_error(
            "getNextSqrtPriceAUp - Next sqrt price greater than max sqrt price",
            CoinErrorCode::CoinAmountMaxExceeded,
        ));
    }

    Ok(next_sqrt_price)
}

/// Get the next sqrt price from give a delta of token_b.
/// `new_sqrt_price = (sqrt_price +(delta_b / liquidity)`
///
/// # Arguments
///
/// * `sqrt_price` - The start sqrt price
/// * `liquidity` - The amount of usable liquidity
/// * `amount` - The amount of token_b
/// * `by_amount_in` - Weather to fixed input
///
/// # Returns
///
/// * The next sqrt price
pub fn get_next_sqrt_price_b_down(sqrt_price: &BigInt, liquidity: &BigInt, amount: &BigInt, by_amount_in: bool) -> Result<BigInt, ClmmpoolsError> {
    let delta_sqrt_price = MathUtil::check_div_round_up_if(&(amount << 64), liquidity, !by_amount_in)?;
    let next_sqrt_price = if by_amount_in {
        sqrt_price + &delta_sqrt_price
    } else {
        sqrt_price - delta_sqrt_price
    };

    if next_sqrt_price < *MIN_SQRT_PRICE || next_sqrt_price > *MAX_SQRT_PRICE {
        return Err(ClmmpoolsError::coin_error(
            "getNextSqrtPriceBDown - Next sqrt price out of bounds",
            CoinErrorCode::SqrtPriceOutOfBounds,
        ));
    }

    Ok(next_sqrt_price)
}

/// Get next sqrt price from input parameter.
///
/// # Arguments
///
/// * `sqrt_price` - The current sqrt price
/// * `liquidity` - The amount of usable liquidity
/// * `amount` - The amount of token to swap
/// * `a_to_b` - The direction of the swap
///
/// # Returns
///
/// * The next sqrt price
pub fn get_next_sqrt_price_from_input(sqrt_price: &BigInt, liquidity: &BigInt, amount: &BigInt, a_to_b: bool) -> Result<BigInt, ClmmpoolsError> {
    if a_to_b {
        get_next_sqrt_price_a_up(sqrt_price, liquidity, amount, true)
    } else {
        get_next_sqrt_price_b_down(sqrt_price, liquidity, amount, true)
    }
}

/// Get the next sqrt price from output parameters.
///
/// # Arguments
///
/// * `sqrt_price` - The current sqrt price
/// * `liquidity` - The amount of usable liquidity
/// * `amount` - The amount of token to swap
/// * `a_to_b` - The direction of the swap
///
/// # Returns
///
/// * The next sqrt price
pub fn get_next_sqrt_price_from_output(sqrt_price: &BigInt, liquidity: &BigInt, amount: &BigInt, a_to_b: bool) -> Result<BigInt, ClmmpoolsError> {
    if a_to_b {
        get_next_sqrt_price_b_down(sqrt_price, liquidity, amount, false)
    } else {
        get_next_sqrt_price_a_up(sqrt_price, liquidity, amount, false)
    }
}

/// Get the amount of delta_a or delta_b from input parameters, and round up result.
///
/// # Arguments
///
/// * `current_sqrt_price` - The current sqrt price
/// * `target_sqrt_price` - The target sqrt price
/// * `liquidity` - The amount of usable liquidity
/// * `a_to_b` - The direction of the swap
///
/// # Returns
///
/// * The delta amount
pub fn get_delta_up_from_input(current_sqrt_price: &BigInt, target_sqrt_price: &BigInt, liquidity: &BigInt, a_to_b: bool) -> BigInt {
    let sqrt_price_diff = if current_sqrt_price > target_sqrt_price {
        current_sqrt_price - target_sqrt_price
    } else {
        target_sqrt_price - current_sqrt_price
    };

    if liquidity <= &*ZERO || sqrt_price_diff == *ZERO {
        return ZERO.clone();
    }

    if a_to_b {
        let numerator = (liquidity * &sqrt_price_diff) << 64;
        let denominator = target_sqrt_price * current_sqrt_price;
        let quotient = &numerator / &denominator;
        let remainder = &numerator % &denominator;
        if remainder > *ZERO {
            quotient + &*ONE
        } else {
            quotient
        }
    } else {
        let product = liquidity * &sqrt_price_diff;
        let should_round_up = (&product & &*U64_MAX) > *ZERO;
        if should_round_up {
            &product >> (64 + 1)
        } else {
            &product >> 64
        }
    }
}

/// Get the amount of delta_a or delta_b from output parameters, and round down result.
///
/// # Arguments
///
/// * `current_sqrt_price` - The current sqrt price
/// * `target_sqrt_price` - The target sqrt price
/// * `liquidity` - The amount of usable liquidity
/// * `a_to_b` - The direction of the swap
///
/// # Returns
///
/// * The delta amount
pub fn get_delta_down_from_output(current_sqrt_price: &BigInt, target_sqrt_price: &BigInt, liquidity: &BigInt, a_to_b: bool) -> BigInt {
    let sqrt_price_diff = if current_sqrt_price > target_sqrt_price {
        current_sqrt_price - target_sqrt_price
    } else {
        target_sqrt_price - current_sqrt_price
    };

    if liquidity <= &*ZERO || sqrt_price_diff == *ZERO {
        return ZERO.clone();
    }

    if a_to_b {
        let product = liquidity * &sqrt_price_diff;
        &product >> 64
    } else {
        let numerator = (liquidity * &sqrt_price_diff) << 64;
        let denominator = target_sqrt_price * current_sqrt_price;
        &numerator / &denominator
    }
}

/// Simulate per step of swap on every tick.
///
/// # Arguments
///
/// * `current_sqrt_price` - The current sqrt price
/// * `target_sqrt_price` - The target sqrt price
/// * `liquidity` - The amount of usable liquidity
/// * `amount` - The amount of token to swap
/// * `fee_rate` - The fee rate
/// * `by_amount_in` - Weather to fixed input
///
/// # Returns
///
/// * The swap step result
pub fn compute_swap_step(
    current_sqrt_price: &BigInt,
    target_sqrt_price: &BigInt,
    liquidity: &BigInt,
    amount: &BigInt,
    fee_rate: &BigInt,
    by_amount_in: bool,
) -> Result<SwapStepResult, ClmmpoolsError> {
    if liquidity == &*ZERO {
        return Ok(SwapStepResult {
            amount_in: ZERO.clone(),
            amount_out: ZERO.clone(),
            next_sqrt_price: target_sqrt_price.clone(),
            fee_amount: ZERO.clone(),
        });
    }

    let a_to_b = current_sqrt_price >= target_sqrt_price;
    let amount_in;
    let amount_out;
    let next_sqrt_price;
    let fee_amount;

    if by_amount_in {
        let amount_remain = MathUtil::check_mul_div_floor(
            amount,
            &(MathUtil::check_unsigned_sub(&FEE_RATE_DENOMINATOR, fee_rate)?),
            &FEE_RATE_DENOMINATOR,
            64,
        )?;

        let max_amount_in = get_delta_up_from_input(current_sqrt_price, target_sqrt_price, liquidity, a_to_b);

        if max_amount_in > amount_remain {
            amount_in = amount_remain.clone();
            fee_amount = MathUtil::check_unsigned_sub(amount, &amount_in)?;
            next_sqrt_price = get_next_sqrt_price_from_input(current_sqrt_price, liquidity, &amount_in, a_to_b)?;
        } else {
            amount_in = max_amount_in.clone();
            fee_amount = MathUtil::check_mul_div_ceil(&max_amount_in, fee_rate, &MathUtil::check_unsigned_sub(&FEE_RATE_DENOMINATOR, fee_rate)?, 64)?;
            next_sqrt_price = target_sqrt_price.clone();
        }

        amount_out = get_delta_down_from_output(current_sqrt_price, &next_sqrt_price, liquidity, a_to_b);
    } else {
        let max_amount_out = get_delta_down_from_output(current_sqrt_price, target_sqrt_price, liquidity, a_to_b);

        if max_amount_out > *amount {
            amount_out = amount.clone();
            next_sqrt_price = get_next_sqrt_price_from_output(current_sqrt_price, liquidity, amount, a_to_b)?;
        } else {
            amount_out = max_amount_out;
            next_sqrt_price = target_sqrt_price.clone();
        }

        amount_in = get_delta_up_from_input(current_sqrt_price, &next_sqrt_price, liquidity, a_to_b);
        fee_amount = MathUtil::check_mul_div_ceil(&amount_in, fee_rate, &MathUtil::check_unsigned_sub(&FEE_RATE_DENOMINATOR, fee_rate)?, 64)?;
    }

    Ok(SwapStepResult {
        amount_in,
        amount_out,
        next_sqrt_price,
        fee_amount,
    })
}

/// Simple representation of clmmpool data
#[derive(Clone, Default)]
pub struct ClmmPoolData {
    pub liquidity: BigInt,
    pub current_tick_index: i32,
    pub current_sqrt_price: BigInt,
    pub fee_rate: BigInt,
    pub fee_growth_global_a: BigInt,
    pub fee_growth_global_b: BigInt,
    pub fee_protocol_coin_a: BigInt,
    pub fee_protocol_coin_b: BigInt,
}

/// Simple representation of tick data
#[derive(Clone)]
pub struct TickData {
    pub index: i32,
    pub sqrt_price: BigInt,
    pub liquidity_net: BigInt,
}

/// Simulate swap by input lots of ticks.
///
/// # Arguments
///
/// * `a_to_b` - The direction of the swap
/// * `by_amount_in` - Weather to fixed input
/// * `amount` - The amount of token to swap
/// * `pool_data` - The pool data
/// * `swap_ticks` - The ticks to swap through
///
/// # Returns
///
/// * The swap result
pub fn compute_swap(
    a_to_b: bool,
    by_amount_in: bool,
    amount: &BigInt,
    pool_data: &ClmmPoolData,
    swap_ticks: &[TickData],
) -> Result<SwapResult, ClmmpoolsError> {
    let mut remainder_amount = amount.clone();
    let mut current_liquidity = pool_data.liquidity.clone();
    let mut current_sqrt_price = pool_data.current_sqrt_price.clone();

    let mut swap_result = SwapResult {
        amount_in: ZERO.clone(),
        amount_out: ZERO.clone(),
        fee_amount: ZERO.clone(),
        ref_amount: ZERO.clone(),
        next_sqrt_price: ZERO.clone(),
        cross_tick_num: 0,
    };

    let mut target_sqrt_price;
    let mut signed_liquidity_change;
    let sqrt_price_limit = SwapUtils::get_default_sqrt_price_limit(a_to_b);

    for tick in swap_ticks {
        if a_to_b && pool_data.current_tick_index < tick.index {
            continue;
        }

        if !a_to_b && pool_data.current_tick_index >= tick.index {
            continue;
        }

        if (a_to_b && sqrt_price_limit > tick.sqrt_price) || (!a_to_b && sqrt_price_limit < tick.sqrt_price) {
            target_sqrt_price = sqrt_price_limit.clone();
        } else {
            target_sqrt_price = tick.sqrt_price.clone();
        }

        let step_result = compute_swap_step(
            &current_sqrt_price,
            &target_sqrt_price,
            &current_liquidity,
            &remainder_amount,
            &pool_data.fee_rate,
            by_amount_in,
        )?;

        if step_result.amount_in != *ZERO {
            remainder_amount = if by_amount_in {
                &remainder_amount - &(&step_result.amount_in + &step_result.fee_amount)
            } else {
                &remainder_amount - &step_result.amount_out
            };
        }

        swap_result.amount_in = &swap_result.amount_in + &step_result.amount_in;
        swap_result.amount_out = &swap_result.amount_out + &step_result.amount_out;
        swap_result.fee_amount = &swap_result.fee_amount + &step_result.fee_amount;

        if step_result.next_sqrt_price == tick.sqrt_price {
            signed_liquidity_change = &tick.liquidity_net * BigInt::from(-1);

            if a_to_b {
                if MathUtil::is_neg(&signed_liquidity_change) {
                    // Convert negative liquidity to u128
                    let as_u128 = if signed_liquidity_change < *ZERO {
                        let pos_value = -&signed_liquidity_change;
                        pos_value & ((BigInt::from(1) << 128) - 1)
                    } else {
                        signed_liquidity_change.clone()
                    };
                    current_liquidity += as_u128;
                } else {
                    current_liquidity += signed_liquidity_change;
                }
            } else if MathUtil::is_neg(&signed_liquidity_change) {
                // Convert negative liquidity to u128
                let as_u128 = if signed_liquidity_change < *ZERO {
                    let pos_value = -&signed_liquidity_change;
                    pos_value & ((BigInt::from(1) << 128) - 1)
                } else {
                    signed_liquidity_change.clone()
                };
                current_liquidity -= as_u128;
            } else {
                current_liquidity -= signed_liquidity_change;
            }

            current_sqrt_price = tick.sqrt_price.clone();
        } else {
            current_sqrt_price = step_result.next_sqrt_price;
        }

        swap_result.cross_tick_num += 1;

        if remainder_amount == *ZERO {
            break;
        }
    }

    swap_result.amount_in = &swap_result.amount_in + &swap_result.fee_amount;
    swap_result.next_sqrt_price = current_sqrt_price;

    Ok(swap_result)
}

/// Estimate liquidity for coin A
///
/// # Arguments
///
/// * `sqrt_price_x` - coin A sqrtprice
/// * `sqrt_price_y` - coin B sqrtprice
/// * `coin_amount` - token amount
///
/// # Returns
///
/// * The estimated liquidity
pub fn estimate_liquidity_for_coin_a(sqrt_price_x: &BigInt, sqrt_price_y: &BigInt, coin_amount: &BigInt) -> BigInt {
    let lower_sqrt_price_x64 = std::cmp::min(sqrt_price_x, sqrt_price_y);
    let upper_sqrt_price_x64 = std::cmp::max(sqrt_price_x, sqrt_price_y);
    let num = MathUtil::from_x64_bn(&(coin_amount * upper_sqrt_price_x64 * lower_sqrt_price_x64));
    let dem = upper_sqrt_price_x64 - lower_sqrt_price_x64;

    if num != *ZERO && dem != *ZERO {
        num / dem
    } else {
        ZERO.clone()
    }
}

/// Estimate liquidity for coin B
///
/// # Arguments
///
/// * `sqrt_price_x` - coin A sqrtprice
/// * `sqrt_price_y` - coin B sqrtprice
/// * `coin_amount` - token amount
///
/// # Returns
///
/// * The estimated liquidity
pub fn estimate_liquidity_for_coin_b(sqrt_price_x: &BigInt, sqrt_price_y: &BigInt, coin_amount: &BigInt) -> BigInt {
    let lower_sqrt_price_x64 = std::cmp::min(sqrt_price_x, sqrt_price_y);
    let upper_sqrt_price_x64 = std::cmp::max(sqrt_price_x, sqrt_price_y);
    let delta = upper_sqrt_price_x64 - lower_sqrt_price_x64;

    if delta != *ZERO {
        (coin_amount << 64) / delta
    } else {
        ZERO.clone()
    }
}

/// Structure for representing liquidity input
pub struct LiquidityInput {
    pub coin_amount_a: BigInt,
    pub coin_amount_b: BigInt,
    pub token_max_a: BigInt,
    pub token_max_b: BigInt,
    pub liquidity_amount: BigInt,
    pub fix_amount_a: bool,
}

pub struct ClmmPoolUtil;

impl ClmmPoolUtil {
    /// Update fee rate.
    ///
    /// # Arguments
    ///
    /// * `clmm` - clmmpool data
    /// * `fee_amount` - fee Amount
    /// * `ref_rate` - ref rate
    /// * `protocol_fee_rate` - protocol fee rate
    /// * `is_coin_a` - is token A
    ///
    /// # Returns
    ///
    /// * The ref fee and updated clmm
    pub fn update_fee_rate(
        clmm: &mut ClmmPoolData,
        fee_amount: &BigInt,
        ref_rate: i64,
        protocol_fee_rate: i64,
        is_coin_a: bool,
    ) -> Result<BigInt, ClmmpoolsError> {
        let protocol_fee = MathUtil::check_mul_div_ceil(fee_amount, &BigInt::from(protocol_fee_rate), &FEE_RATE_DENOMINATOR, 64)?;

        let ref_fee = if ref_rate == 0 {
            ZERO.clone()
        } else {
            MathUtil::check_mul_div_floor(fee_amount, &BigInt::from(ref_rate), &FEE_RATE_DENOMINATOR, 64)?
        };

        let pool_fee = fee_amount - &protocol_fee - &ref_fee;

        if is_coin_a {
            clmm.fee_protocol_coin_a = &clmm.fee_protocol_coin_a + &protocol_fee;
        } else {
            clmm.fee_protocol_coin_b = &clmm.fee_protocol_coin_b + &protocol_fee;
        }

        if pool_fee == *ZERO || clmm.liquidity == *ZERO {
            return Ok(ref_fee);
        }

        let growth_fee = (pool_fee << 64) / &clmm.liquidity;

        if is_coin_a {
            clmm.fee_growth_global_a = &clmm.fee_growth_global_a + &growth_fee;
        } else {
            clmm.fee_growth_global_b = &clmm.fee_growth_global_b + &growth_fee;
        }

        Ok(ref_fee)
    }

    /// Get token amount from liquidity.
    ///
    /// # Arguments
    ///
    /// * `liquidity` - liquidity
    /// * `cur_sqrt_price` - Pool current sqrt price
    /// * `lower_sqrt_price` - position lower sqrt price
    /// * `upper_sqrt_price` - position upper sqrt price
    /// * `round_up` - is round up
    ///
    /// # Returns
    ///
    /// * The coin amounts
    pub fn get_coin_amount_from_liquidity(
        liquidity: &BigInt,
        cur_sqrt_price: &BigInt,
        lower_sqrt_price: &BigInt,
        upper_sqrt_price: &BigInt,
        round_up: bool,
    ) -> CoinAmounts {
        let liq = BigDecimal::from_str(&liquidity.to_string()).unwrap();
        let cur_sqrt_price_str = BigDecimal::from_str(&cur_sqrt_price.to_string()).unwrap();
        let lower_price_str = BigDecimal::from_str(&lower_sqrt_price.to_string()).unwrap();
        let upper_price_str = BigDecimal::from_str(&upper_sqrt_price.to_string()).unwrap();

        let coin_a;
        let coin_b;

        if cur_sqrt_price < lower_sqrt_price {
            // Price is below the position's range - only token A
            let term1 = MathUtil::to_x64_decimal(&liq);
            let term2 = &upper_price_str - &lower_price_str;
            let term3 = &lower_price_str * &upper_price_str;
            coin_a = &term1 * &term2 / &term3;
            coin_b = BigDecimal::from(0);
        } else if cur_sqrt_price < upper_sqrt_price {
            // Price is within the position's range - both tokens
            let term1 = MathUtil::to_x64_decimal(&liq);
            let term2 = &upper_price_str - &cur_sqrt_price_str;
            let term3 = &cur_sqrt_price_str * &upper_price_str;
            coin_a = &term1 * &term2 / &term3;

            let term1 = MathUtil::from_x64_decimal(&liq);
            let term2 = &cur_sqrt_price_str - &lower_price_str;
            coin_b = &term1 * &term2;
        } else {
            // Price is above the position's range - only token B
            coin_a = BigDecimal::from(0);
            let term1 = MathUtil::from_x64_decimal(&liq);
            let term2 = &upper_price_str - &lower_price_str;
            coin_b = &term1 * &term2;
        }

        if round_up {
            CoinAmounts {
                coin_a: BigInt::from_str(&coin_a.with_scale(0).round(0).to_string()).unwrap(),
                coin_b: BigInt::from_str(&coin_b.with_scale(0).round(0).to_string()).unwrap(),
            }
        } else {
            CoinAmounts {
                coin_a: BigInt::from_str(&coin_a.with_scale(0).round(-1).to_string()).unwrap(),
                coin_b: BigInt::from_str(&coin_b.with_scale(0).round(-1).to_string()).unwrap(),
            }
        }
    }

    /// Estimate liquidity and token amount from one amounts
    ///
    /// # Arguments
    ///
    /// * `lower_tick` - lower tick
    /// * `upper_tick` - upper tick
    /// * `coin_amount` - token amount
    /// * `is_coin_a` - is token A
    /// * `round_up` - is round up
    /// * `slippage` - slippage percentage
    /// * `cur_sqrt_price` - current sqrt price
    ///
    /// # Returns
    ///
    /// * The liquidity input
    pub fn est_liquidity_and_coin_amount_from_one_amounts(
        lower_tick: i32,
        upper_tick: i32,
        coin_amount: &BigInt,
        is_coin_a: bool,
        round_up: bool,
        slippage: BigDecimal,
        cur_sqrt_price: &BigInt,
    ) -> Result<LiquidityInput, ClmmpoolsError> {
        let current_tick = TickMath::sqrt_price_x64_to_tick_index(cur_sqrt_price)?;
        let lower_sqrt_price = TickMath::tick_index_to_sqrt_price_x64(lower_tick);
        let upper_sqrt_price = TickMath::tick_index_to_sqrt_price_x64(upper_tick);

        let liquidity;
        if current_tick < lower_tick {
            if !is_coin_a {
                return Err(ClmmpoolsError::math_error(
                    "lower tick cannot calculate liquidity by coinB",
                    MathErrorCode::NotSupportedThisCoin,
                ));
            }
            liquidity = estimate_liquidity_for_coin_a(&lower_sqrt_price, &upper_sqrt_price, coin_amount);
        } else if current_tick > upper_tick {
            if is_coin_a {
                return Err(ClmmpoolsError::math_error(
                    "upper tick cannot calculate liquidity by coinA",
                    MathErrorCode::NotSupportedThisCoin,
                ));
            }
            liquidity = estimate_liquidity_for_coin_b(&upper_sqrt_price, &lower_sqrt_price, coin_amount);
        } else if is_coin_a {
            liquidity = estimate_liquidity_for_coin_a(cur_sqrt_price, &upper_sqrt_price, coin_amount);
        } else {
            liquidity = estimate_liquidity_for_coin_b(cur_sqrt_price, &lower_sqrt_price, coin_amount);
        }

        let coin_amounts = Self::get_coin_amount_from_liquidity(&liquidity, cur_sqrt_price, &lower_sqrt_price, &upper_sqrt_price, round_up);

        let slippage_factor = if round_up {
            BigDecimal::one() + slippage
        } else {
            BigDecimal::one() - slippage
        };

        let token_limit_a = BigDecimal::from_str(&coin_amounts.coin_a.to_string()).unwrap() * &slippage_factor;
        let token_limit_b = BigDecimal::from_str(&coin_amounts.coin_b.to_string()).unwrap() * &slippage_factor;

        let token_max_a = if round_up {
            ceil_bigdecimal(&token_limit_a).into_bigint_and_exponent().0
        } else {
            floor_bigdecimal(&token_limit_a).into_bigint_and_exponent().0
        };

        let token_max_b = if round_up {
            ceil_bigdecimal(&token_limit_b).into_bigint_and_exponent().0
        } else {
            floor_bigdecimal(&token_limit_b).into_bigint_and_exponent().0
        };

        Ok(LiquidityInput {
            coin_amount_a: coin_amounts.coin_a,
            coin_amount_b: coin_amounts.coin_b,
            token_max_a,
            token_max_b,
            liquidity_amount: liquidity,
            fix_amount_a: is_coin_a,
        })
    }

    /// Estimate liquidity from token amounts
    ///
    /// # Arguments
    ///
    /// * `cur_sqrt_price` - current sqrt price
    /// * `lower_tick` - lower tick
    /// * `upper_tick` - upper tick
    /// * `token_amount` - token amount
    ///
    /// # Returns
    ///
    /// * The estimated liquidity
    pub fn estimate_liquidity_from_coin_amounts(
        cur_sqrt_price: &BigInt,
        lower_tick: i32,
        upper_tick: i32,
        token_amount: &CoinAmounts,
    ) -> Result<BigInt, ClmmpoolsError> {
        if lower_tick > upper_tick {
            return Err(ClmmpoolsError::math_error(
                "lower tick cannot be greater than lower tick",
                MathErrorCode::InvalidTwoTickIndex,
            ));
        }

        let curr_tick = TickMath::sqrt_price_x64_to_tick_index(cur_sqrt_price)?;
        let lower_sqrt_price = TickMath::tick_index_to_sqrt_price_x64(lower_tick);
        let upper_sqrt_price = TickMath::tick_index_to_sqrt_price_x64(upper_tick);

        if curr_tick < lower_tick {
            return Ok(estimate_liquidity_for_coin_a(&lower_sqrt_price, &upper_sqrt_price, &token_amount.coin_a));
        }

        if curr_tick >= upper_tick {
            return Ok(estimate_liquidity_for_coin_b(&upper_sqrt_price, &lower_sqrt_price, &token_amount.coin_b));
        }

        let estimate_liquidity_amount_a = estimate_liquidity_for_coin_a(cur_sqrt_price, &upper_sqrt_price, &token_amount.coin_a);
        let estimate_liquidity_amount_b = estimate_liquidity_for_coin_b(cur_sqrt_price, &lower_sqrt_price, &token_amount.coin_b);

        Ok(std::cmp::min(estimate_liquidity_amount_a, estimate_liquidity_amount_b))
    }

    /// Estimate coin amounts from total amount
    ///
    /// # Arguments
    ///
    /// * `lower_tick` - lower tick
    /// * `upper_tick` - upper tick
    /// * `cur_sqrt_price` - current sqrt price
    /// * `total_amount` - total amount of investment
    /// * `token_price_a` - token A price
    /// * `token_price_b` - token B price
    ///
    /// # Returns
    ///
    /// * The estimated amounts of token A and B
    pub fn est_coin_amounts_from_total_amount(
        lower_tick: i32,
        upper_tick: i32,
        cur_sqrt_price: &BigInt,
        total_amount: &str,
        token_price_a: &str,
        token_price_b: &str,
    ) -> (BigDecimal, BigDecimal) {
        let ratio = Self::calculate_deposit_ratio_fix_token_a(lower_tick, upper_tick, cur_sqrt_price).unwrap();

        let total_amount_dec = BigDecimal::from_str(total_amount).unwrap();
        let token_price_a_dec = BigDecimal::from_str(token_price_a).unwrap();
        let token_price_b_dec = BigDecimal::from_str(token_price_b).unwrap();

        let amount_a = &total_amount_dec * &ratio.0 / &token_price_a_dec;
        let amount_b = &total_amount_dec * &ratio.1 / &token_price_b_dec;

        (amount_a, amount_b)
    }

    /// Calculate deposit ratio fixing token A
    ///
    /// # Arguments
    ///
    /// * `lower_tick` - lower tick
    /// * `upper_tick` - upper tick
    /// * `cur_sqrt_price` - current sqrt price
    ///
    /// # Returns
    ///
    /// * The ratio of token A and B
    pub fn calculate_deposit_ratio_fix_token_a(lower_tick: i32, upper_tick: i32, cur_sqrt_price: &BigInt) -> Result<(BigDecimal, BigDecimal), ClmmpoolsError> {
        let coin_amount_a = BigInt::from(100000000); // Fixed test amount

        let liquidity_input =
            Self::est_liquidity_and_coin_amount_from_one_amounts(lower_tick, upper_tick, &coin_amount_a, true, true, BigDecimal::zero(), cur_sqrt_price)?;

        let curr_price = TickMath::sqrt_price_x64_to_price(cur_sqrt_price, 0, 0);

        let transform_amount_b = BigDecimal::from_str(&coin_amount_a.to_string()).unwrap() * curr_price;
        let total_amount = &transform_amount_b + BigDecimal::from_str(&liquidity_input.coin_amount_b.to_string()).unwrap();

        let ratio_a = &transform_amount_b / &total_amount;
        let ratio_b = BigDecimal::from_str(&liquidity_input.coin_amount_b.to_string()).unwrap() / total_amount;

        Ok((ratio_a, ratio_b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use std::str::FromStr;

    #[test]
    fn test_get_delta_a() {
        // Test with simple values
        let sqrt_price0 = BigInt::from(10);
        let sqrt_price1 = BigInt::from(20);
        let liquidity = BigInt::from(1000);

        // Test round up
        let result_round_up = get_delta_a(&sqrt_price0, &sqrt_price1, &liquidity, true);

        // Test round down
        let result_round_down = get_delta_a(&sqrt_price0, &sqrt_price1, &liquidity, false);

        // Verify round up >= round down
        assert!(result_round_up >= result_round_down);
    }

    #[test]
    fn test_get_delta_b() {
        // Test with simple values
        let sqrt_price0 = BigInt::from(10);
        let sqrt_price1 = BigInt::from(20);
        let liquidity = BigInt::from(1000);

        // Test round up
        let result_round_up = get_delta_b(&sqrt_price0, &sqrt_price1, &liquidity, true).unwrap();

        // Test round down
        let result_round_down = get_delta_b(&sqrt_price0, &sqrt_price1, &liquidity, false).unwrap();

        // Verify round up >= round down
        assert!(result_round_up >= result_round_down);

        // Test with zero liquidity
        let zero_liquidity = get_delta_b(&sqrt_price0, &sqrt_price1, &ZERO, true).unwrap();
        assert_eq!(zero_liquidity, *ZERO);

        // Test with zero price diff
        let zero_diff = get_delta_b(&sqrt_price0, &sqrt_price0, &liquidity, true).unwrap();
        assert_eq!(zero_diff, *ZERO);
    }

    #[test]
    fn test_get_next_sqrt_price_a_up() {
        // Test with simple values
        let sqrt_price = BigInt::from(100000); // Non-zero
        let liquidity = BigInt::from(1000);
        let amount = BigInt::from(500);

        // Test by amount in
        let _result_by_amount_in = get_next_sqrt_price_a_up(&sqrt_price, &liquidity, &amount, true).unwrap();

        // Test by amount out
        let _result_by_amount_out = get_next_sqrt_price_a_up(&sqrt_price, &liquidity, &amount, false).unwrap();

        // Zero amount should return the same sqrt price
        let result_zero_amount = get_next_sqrt_price_a_up(&sqrt_price, &liquidity, &ZERO, true).unwrap();
        assert_eq!(result_zero_amount, sqrt_price);
    }

    #[test]
    fn test_delta_up_from_input_and_delta_down_from_output() {
        // Test with simple values
        let current_sqrt_price = BigInt::from(100000);
        let target_sqrt_price = BigInt::from(90000);
        let liquidity = BigInt::from(1000);

        // Test for A to B
        let delta_up_a_to_b = get_delta_up_from_input(&current_sqrt_price, &target_sqrt_price, &liquidity, true);
        let delta_down_a_to_b = get_delta_down_from_output(&current_sqrt_price, &target_sqrt_price, &liquidity, true);

        // Verify delta up >= delta down (due to rounding)
        assert!(delta_up_a_to_b >= delta_down_a_to_b);

        // Test for B to A
        let delta_up_b_to_a = get_delta_up_from_input(&current_sqrt_price, &target_sqrt_price, &liquidity, false);
        let delta_down_b_to_a = get_delta_down_from_output(&current_sqrt_price, &target_sqrt_price, &liquidity, false);

        // Verify delta up >= delta down (due to rounding)
        assert!(delta_up_b_to_a >= delta_down_b_to_a);

        // Test with zero liquidity
        let zero_liquidity_up = get_delta_up_from_input(&current_sqrt_price, &target_sqrt_price, &ZERO, true);
        let zero_liquidity_down = get_delta_down_from_output(&current_sqrt_price, &target_sqrt_price, &ZERO, true);
        assert_eq!(zero_liquidity_up, *ZERO);
        assert_eq!(zero_liquidity_down, *ZERO);

        // Test with zero price diff
        let zero_diff_up = get_delta_up_from_input(&current_sqrt_price, &current_sqrt_price, &liquidity, true);
        let zero_diff_down = get_delta_down_from_output(&current_sqrt_price, &current_sqrt_price, &liquidity, true);
        assert_eq!(zero_diff_up, *ZERO);
        assert_eq!(zero_diff_down, *ZERO);
    }

    #[test]
    fn test_compute_swap_step() {
        // Test with simple values
        let current_sqrt_price = BigInt::from(100000);
        let target_sqrt_price = BigInt::from(90000);
        let liquidity = BigInt::from(1000);
        let amount = BigInt::from(500);
        let fee_rate = BigInt::from(3000); // 0.3%

        // Test by amount in
        let _step_in = compute_swap_step(&current_sqrt_price, &target_sqrt_price, &liquidity, &amount, &fee_rate, true).unwrap();

        // Test by amount out
        let _step_out = compute_swap_step(&current_sqrt_price, &target_sqrt_price, &liquidity, &amount, &fee_rate, false).unwrap();

        // Test with zero liquidity
        let zero_liquidity = compute_swap_step(&current_sqrt_price, &target_sqrt_price, &ZERO, &amount, &fee_rate, true).unwrap();

        assert_eq!(zero_liquidity.next_sqrt_price, target_sqrt_price);
        assert_eq!(zero_liquidity.amount_in, *ZERO);
        assert_eq!(zero_liquidity.amount_out, *ZERO);
        assert_eq!(zero_liquidity.fee_amount, *ZERO);
    }

    #[test]
    fn test_estimate_liquidity_for_coins() {
        // Test with simple values
        let sqrt_price_x = BigInt::from(100000);
        let sqrt_price_y = BigInt::from(120000);
        let coin_amount = BigInt::from(1000);

        // Test for coin A
        let _liquidity_a = estimate_liquidity_for_coin_a(&sqrt_price_x, &sqrt_price_y, &coin_amount);

        // Test for coin B
        let _liquidity_b = estimate_liquidity_for_coin_b(&sqrt_price_x, &sqrt_price_y, &coin_amount);

        // Edge cases
        let zero_amount_a = estimate_liquidity_for_coin_a(&sqrt_price_x, &sqrt_price_y, &ZERO);
        let zero_amount_b = estimate_liquidity_for_coin_b(&sqrt_price_x, &sqrt_price_y, &ZERO);

        assert_eq!(zero_amount_a, *ZERO);
        assert_eq!(zero_amount_b, *ZERO);

        let equal_prices_a = estimate_liquidity_for_coin_a(&sqrt_price_x, &sqrt_price_x, &coin_amount);
        let equal_prices_b = estimate_liquidity_for_coin_b(&sqrt_price_x, &sqrt_price_x, &coin_amount);

        assert_eq!(equal_prices_a, *ZERO);
        assert_eq!(equal_prices_b, *ZERO);
    }

    #[test]
    fn test_get_coin_amount_from_liquidity() {
        // Test with simple values
        let liquidity = BigInt::from(1000000);
        let current_sqrt_price = BigInt::from_str("79228162514264337593543950336").unwrap(); // 1.0 in x64
        let lower_sqrt_price = BigInt::from_str("79228162514264337593543950336").unwrap() * 9 / 10; // 0.9 in x64
        let upper_sqrt_price = BigInt::from_str("79228162514264337593543950336").unwrap() * 11 / 10; // 1.1 in x64

        // Price is in range
        let in_range = ClmmPoolUtil::get_coin_amount_from_liquidity(&liquidity, &current_sqrt_price, &lower_sqrt_price, &upper_sqrt_price, false);

        // Both coin amounts should be positive
        assert!(in_range.coin_a > *ZERO);
        assert!(in_range.coin_b > *ZERO);

        // Price is below range
        let below_range = ClmmPoolUtil::get_coin_amount_from_liquidity(
            &liquidity,
            &(lower_sqrt_price.clone() * BigInt::from(8) / BigInt::from(10)), // 0.72 in x64
            &lower_sqrt_price,
            &upper_sqrt_price,
            false,
        );

        // Only coin A should be positive
        assert!(below_range.coin_a > *ZERO);
        assert_eq!(below_range.coin_b, *ZERO);

        // Price is above range
        let above_range = ClmmPoolUtil::get_coin_amount_from_liquidity(
            &liquidity,
            &(upper_sqrt_price.clone() * BigInt::from(12) / BigInt::from(10)), // 1.32 in x64
            &lower_sqrt_price,
            &upper_sqrt_price,
            false,
        );

        // Only coin B should be positive
        assert_eq!(above_range.coin_a, *ZERO);
        assert!(above_range.coin_b > *ZERO);

        // Test rounding
        let round_up = ClmmPoolUtil::get_coin_amount_from_liquidity(&liquidity, &current_sqrt_price, &lower_sqrt_price, &upper_sqrt_price, true);

        let round_down = ClmmPoolUtil::get_coin_amount_from_liquidity(&liquidity, &current_sqrt_price, &lower_sqrt_price, &upper_sqrt_price, false);

        // Rounded up amounts should be >= rounded down amounts
        assert!(round_up.coin_a >= round_down.coin_a);
        assert!(round_up.coin_b >= round_down.coin_b);
    }
}
