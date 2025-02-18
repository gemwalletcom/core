use num_bigint::BigInt;
use num_traits::Signed;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct PoolData {
    pub current_tick_index: i32,
    pub fee_rate: BigInt,
}

#[derive(Debug, Clone)]
pub struct TickData {
    pub index: i32,
    pub sqrt_price: BigInt,
    pub liquidity_net: BigInt,
}

#[derive(Debug, Clone, Default)]
pub struct SwapResult {
    pub amount_in: BigInt,
    pub amount_out: BigInt,
    pub fee_amount: BigInt,
    pub ref_amount: BigInt,
    pub next_sqrt_price: BigInt,
    pub cross_tick_num: u64,
}

#[derive(Debug, Clone)]
pub struct SwapStepResult {
    pub amount_in: BigInt,
    pub amount_out: BigInt,
    pub fee_amount: BigInt,
    pub next_sqrt_price: BigInt,
}

pub fn compute_swap(
    pool_data: &PoolData,
    swap_ticks: Vec<TickData>,
    current_sqrt_price: BigInt,
    current_liquidity: BigInt,
    remaining_amount: BigInt,
    a_to_b: bool,
    by_amount_in: bool,
) -> SwapResult {
    let mut swap_result = SwapResult::default();
    let mut current_sqrt_price = current_sqrt_price;
    let mut current_liquidity = current_liquidity;
    let mut remainder_amount = remaining_amount;

    let sqrt_price_limit = if a_to_b {
        BigInt::from(4295048016u64) // MIN_SQRT_PRICE
    } else {
        BigInt::from_str("79226673515401279992447579055").unwrap() // MAX_SQRT_PRICE
    };

    for tick in swap_ticks {
        if (a_to_b && pool_data.current_tick_index < tick.index) || (!a_to_b && pool_data.current_tick_index >= tick.index) {
            continue;
        }

        let target_sqrt_price = if (a_to_b && sqrt_price_limit > tick.sqrt_price) || (!a_to_b && sqrt_price_limit < tick.sqrt_price) {
            sqrt_price_limit.clone()
        } else {
            tick.sqrt_price.clone()
        };

        let step_result = compute_swap_step(
            &current_sqrt_price,
            &target_sqrt_price,
            &current_liquidity,
            &remainder_amount,
            &pool_data.fee_rate,
            by_amount_in,
        );

        if step_result.amount_in != BigInt::from(0) {
            remainder_amount = if by_amount_in {
                &remainder_amount - (&step_result.amount_in + &step_result.fee_amount)
            } else {
                &remainder_amount - &step_result.amount_out
            };
        }

        swap_result.amount_in += &step_result.amount_in;
        swap_result.amount_out += &step_result.amount_out;
        swap_result.fee_amount += &step_result.fee_amount;

        if step_result.next_sqrt_price == tick.sqrt_price {
            let signed_liquidity_change = -&tick.liquidity_net;
            current_liquidity = if a_to_b {
                if signed_liquidity_change < BigInt::from(0) {
                    &current_liquidity + signed_liquidity_change.abs()
                } else {
                    &current_liquidity + signed_liquidity_change
                }
            } else if signed_liquidity_change < BigInt::from(0) {
                &current_liquidity - signed_liquidity_change.abs()
            } else {
                &current_liquidity - signed_liquidity_change
            };
            current_sqrt_price = tick.sqrt_price;
        } else {
            current_sqrt_price = step_result.next_sqrt_price;
        }

        swap_result.cross_tick_num += 1;

        if remainder_amount == BigInt::from(0) {
            break;
        }
    }

    swap_result.amount_in += &swap_result.fee_amount;
    swap_result.next_sqrt_price = current_sqrt_price;
    swap_result
}

fn compute_swap_step(
    current_sqrt_price: &BigInt,
    target_sqrt_price: &BigInt,
    liquidity: &BigInt,
    amount_remaining: &BigInt,
    fee_rate: &BigInt,
    by_amount_in: bool,
) -> SwapStepResult {
    if liquidity == &BigInt::from(0) {
        return SwapStepResult {
            amount_in: BigInt::from(0),
            amount_out: BigInt::from(0),
            fee_amount: BigInt::from(0),
            next_sqrt_price: target_sqrt_price.clone(),
        };
    }

    let amount_delta = calculate_amount_delta(liquidity, current_sqrt_price, target_sqrt_price, true, current_sqrt_price > target_sqrt_price);

    let (amount_in, amount_out, next_sqrt_price) = if by_amount_in {
        let fee_factor = BigInt::from(1000000) - fee_rate;
        let amount_remaining_less_fee = amount_remaining * &fee_factor / BigInt::from(1000000);

        if amount_remaining_less_fee >= amount_delta {
            (
                amount_delta.clone(),
                calculate_amount_delta(liquidity, current_sqrt_price, target_sqrt_price, false, current_sqrt_price < target_sqrt_price),
                target_sqrt_price.clone(),
            )
        } else {
            let next_price = calculate_next_sqrt_price(current_sqrt_price, liquidity, &amount_remaining_less_fee, true);
            (
                amount_remaining_less_fee,
                calculate_amount_delta(liquidity, current_sqrt_price, &next_price, false, current_sqrt_price < &next_price),
                next_price,
            )
        }
    } else if amount_remaining >= &amount_delta {
        (
            calculate_amount_delta(liquidity, current_sqrt_price, target_sqrt_price, true, current_sqrt_price > target_sqrt_price),
            amount_delta,
            target_sqrt_price.clone(),
        )
    } else {
        let next_price = calculate_next_sqrt_price(current_sqrt_price, liquidity, amount_remaining, false);
        (
            calculate_amount_delta(liquidity, current_sqrt_price, &next_price, true, current_sqrt_price > &next_price),
            amount_remaining.clone(),
            next_price,
        )
    };

    let fee_amount = if amount_in == BigInt::from(0) {
        BigInt::from(0)
    } else {
        &amount_in * fee_rate / BigInt::from(1000000)
    };

    SwapStepResult {
        amount_in,
        amount_out,
        fee_amount,
        next_sqrt_price,
    }
}

fn calculate_amount_delta(liquidity: &BigInt, sqrt_price_a: &BigInt, sqrt_price_b: &BigInt, round_up: bool, is_token_0: bool) -> BigInt {
    if is_token_0 {
        let numerator = (liquidity * sqrt_price_b * sqrt_price_a) << 64;
        let denominator = sqrt_price_b * sqrt_price_a;
        if round_up {
            (numerator + denominator.clone() - BigInt::from(1)) / denominator
        } else {
            numerator / denominator
        }
    } else {
        let delta = sqrt_price_b - sqrt_price_a;
        let amount = (liquidity * delta) >> 64;
        if round_up {
            amount + BigInt::from(1)
        } else {
            amount
        }
    }
}

fn calculate_next_sqrt_price(sqrt_price: &BigInt, liquidity: &BigInt, amount: &BigInt, by_amount_in: bool) -> BigInt {
    if by_amount_in {
        let numerator = liquidity << 64;
        let denominator = liquidity + ((amount.clone() << 64) / sqrt_price);
        (numerator * sqrt_price) / (denominator << 64)
    } else {
        let product = liquidity * sqrt_price;
        let mut diff: BigInt = amount << 64;
        diff = diff.abs();
        if amount < &BigInt::from(0) {
            (product + diff - BigInt::from(1)) / liquidity
        } else {
            product / (liquidity + diff)
        }
    }
}
