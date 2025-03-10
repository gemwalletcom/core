use super::constants::*;
use super::error::ErrorCode;
use super::math::*;
use super::{ClmmPoolData, TickData};
use num_bigint::BigInt;

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
    pub next_sqrt_price: BigInt,
    pub cross_tick_num: usize,
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
pub fn get_next_sqrt_price_a_up(sqrt_price: &BigInt, liquidity: &BigInt, amount: &BigInt, by_amount_in: bool) -> Result<BigInt, ErrorCode> {
    if amount == &*ZERO {
        return Ok(sqrt_price.clone());
    }

    let numerator = check_mul_shift_left(sqrt_price, liquidity, 64, 256)?;
    let liquidity_shl64 = liquidity << 64;
    let product = check_mul(sqrt_price, amount, 256)?;

    if !by_amount_in && liquidity_shl64 <= product {
        // Unable to divide liquidityShl64 by product
        return Err(ErrorCode::DivideByZero);
    }

    let next_sqrt_price = if by_amount_in {
        check_div_round_up_if(&numerator, &(liquidity_shl64 + product), true)?
    } else {
        check_div_round_up_if(&numerator, &(liquidity_shl64 - product), true)?
    };

    if next_sqrt_price < *MIN_SQRT_PRICE {
        return Err(ErrorCode::CoinAmountMinSubceeded);
    }

    if next_sqrt_price > *MAX_SQRT_PRICE {
        return Err(ErrorCode::CoinAmountMaxExceeded);
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
pub fn get_next_sqrt_price_b_down(sqrt_price: &BigInt, liquidity: &BigInt, amount: &BigInt, by_amount_in: bool) -> Result<BigInt, ErrorCode> {
    let delta_sqrt_price = check_div_round_up_if(&(amount << 64), liquidity, !by_amount_in)?;
    let next_sqrt_price = if by_amount_in {
        sqrt_price + &delta_sqrt_price
    } else {
        sqrt_price - delta_sqrt_price
    };

    if next_sqrt_price < *MIN_SQRT_PRICE || next_sqrt_price > *MAX_SQRT_PRICE {
        return Err(ErrorCode::SqrtPriceOutOfBounds);
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
pub fn get_next_sqrt_price_from_input(sqrt_price: &BigInt, liquidity: &BigInt, amount: &BigInt, a_to_b: bool) -> Result<BigInt, ErrorCode> {
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
pub fn get_next_sqrt_price_from_output(sqrt_price: &BigInt, liquidity: &BigInt, amount: &BigInt, a_to_b: bool) -> Result<BigInt, ErrorCode> {
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
) -> Result<SwapStepResult, ErrorCode> {
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
        let amount_remain = check_mul_div_floor(amount, &(check_unsigned_sub(&FEE_RATE_DENOMINATOR, fee_rate)?), &FEE_RATE_DENOMINATOR, 64)?;

        let max_amount_in = get_delta_up_from_input(current_sqrt_price, target_sqrt_price, liquidity, a_to_b);

        if max_amount_in > amount_remain {
            amount_in = amount_remain.clone();
            fee_amount = check_unsigned_sub(amount, &amount_in)?;
            next_sqrt_price = get_next_sqrt_price_from_input(current_sqrt_price, liquidity, &amount_in, a_to_b)?;
        } else {
            amount_in = max_amount_in.clone();
            fee_amount = check_mul_div_ceil(&max_amount_in, fee_rate, &check_unsigned_sub(&FEE_RATE_DENOMINATOR, fee_rate)?, 64)?;
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
        fee_amount = check_mul_div_ceil(&amount_in, fee_rate, &check_unsigned_sub(&FEE_RATE_DENOMINATOR, fee_rate)?, 64)?;
    }

    Ok(SwapStepResult {
        amount_in,
        amount_out,
        next_sqrt_price,
        fee_amount,
    })
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
pub fn compute_swap(a_to_b: bool, by_amount_in: bool, amount: &BigInt, pool_data: &ClmmPoolData, swap_ticks: &[TickData]) -> Result<SwapResult, ErrorCode> {
    let mut remainder_amount = amount.clone();
    let mut current_liquidity = pool_data.liquidity.clone();
    let mut current_sqrt_price = pool_data.current_sqrt_price.clone();

    let mut swap_result = SwapResult {
        amount_in: ZERO.clone(),
        amount_out: ZERO.clone(),
        fee_amount: ZERO.clone(),
        next_sqrt_price: ZERO.clone(),
        cross_tick_num: 0,
    };

    let mut target_sqrt_price;
    let mut signed_liquidity_change;
    let sqrt_price_limit = get_default_sqrt_price_limit(a_to_b);

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
                if is_neg(&signed_liquidity_change) {
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
            } else if is_neg(&signed_liquidity_change) {
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_compute_swap() {}

    #[test]
    fn test_compute_swap_step() {}
}
