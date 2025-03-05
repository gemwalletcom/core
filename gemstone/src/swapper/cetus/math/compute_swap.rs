use super::swap::{get_lower_sqrt_price_from_coin_a, get_upper_sqrt_price_from_coin_b};
use super::utils::MathUtil;
use num_bigint::BigInt;
use num_traits::Zero;

/// Represents the result of a swap computation
pub struct SwapResult {
    /// The new sqrt price after the swap
    pub next_sqrt_price: BigInt,
    /// The amount of token A swapped
    pub amount_a: BigInt,
    /// The amount of token B swapped
    pub amount_b: BigInt,
    /// The fee amount in the input token
    pub fee_amount: BigInt,
}

/// Computes the result of swapping along a single tick price range
pub fn compute_swap_step(
    sqrt_price_current: &BigInt,
    sqrt_price_target: &BigInt,
    liquidity: &BigInt,
    amount_remaining: &BigInt,
    fee_rate: &BigInt,
    amount_specified_is_input: bool,
    a2b: bool,
) -> SwapResult {
    let _sqrt_price_start = sqrt_price_current.clone();
    let mut sqrt_price_next;
    let amount_in: BigInt;
    let amount_out: BigInt;
    let mut fee_amount = BigInt::zero();

    if a2b {
        // Swap token A for token B (A is input, B is output)

        // Calculate the price change based on the amount and direction
        if amount_specified_is_input {
            // Trading A for B, given exact input A
            let amount_remaining_less_fee = amount_remaining - (amount_remaining * fee_rate) / BigInt::from(1_000_000);

            sqrt_price_next = get_lower_sqrt_price_from_coin_a(&amount_remaining_less_fee, liquidity, sqrt_price_current);

            // Ensure we don't go beyond the target price
            if sqrt_price_next < *sqrt_price_target {
                sqrt_price_next = sqrt_price_target.clone();
            }
        } else {
            // Trading A for B, given exact output B
            sqrt_price_next = get_upper_sqrt_price_from_coin_b(amount_remaining, liquidity, sqrt_price_current);

            // Ensure we don't go beyond the target price
            if sqrt_price_next < *sqrt_price_target {
                sqrt_price_next = sqrt_price_target.clone();
            }
        }

        // Calculate amounts based on the price change
        amount_in = compute_amount_a_delta(liquidity, &sqrt_price_next, sqrt_price_current, true);

        amount_out = compute_amount_b_delta(liquidity, &sqrt_price_next, sqrt_price_current, false);

        // Compute fees if input amount is specified
        if amount_specified_is_input {
            fee_amount = (amount_in.clone() * fee_rate) / (BigInt::from(1_000_000) - fee_rate);
        }
    } else {
        // Swap token B for token A (B is input, A is output)

        // Calculate the price change based on the amount and direction
        if amount_specified_is_input {
            // Trading B for A, given exact input B
            let amount_remaining_less_fee = amount_remaining - (amount_remaining * fee_rate) / BigInt::from(1_000_000);

            sqrt_price_next = get_upper_sqrt_price_from_coin_b(&amount_remaining_less_fee, liquidity, sqrt_price_current);

            // Ensure we don't go beyond the target price
            if sqrt_price_next > *sqrt_price_target {
                sqrt_price_next = sqrt_price_target.clone();
            }
        } else {
            // Trading B for A, given exact output A
            sqrt_price_next = get_lower_sqrt_price_from_coin_a(amount_remaining, liquidity, sqrt_price_current);

            // Ensure we don't go beyond the target price
            if sqrt_price_next > *sqrt_price_target {
                sqrt_price_next = sqrt_price_target.clone();
            }
        }

        // Calculate amounts based on the price change
        amount_in = compute_amount_b_delta(liquidity, &sqrt_price_next, sqrt_price_current, true);
        amount_out = compute_amount_a_delta(liquidity, &sqrt_price_next, sqrt_price_current, false);

        // Compute fees if input amount is specified
        if amount_specified_is_input {
            fee_amount = (&amount_in * fee_rate) / (BigInt::from(1_000_000) - fee_rate);
        }
    }

    // Return the computed swap result
    SwapResult {
        next_sqrt_price: sqrt_price_next,
        amount_a: if a2b { amount_in.clone() } else { amount_out.clone() },
        amount_b: if a2b { amount_out } else { amount_in },
        fee_amount,
    }
}

/// Compute the amount of token A corresponding to a given change in sqrt price
pub fn compute_amount_a_delta(liquidity: &BigInt, sqrt_price_next: &BigInt, sqrt_price_current: &BigInt, round_up: bool) -> BigInt {
    let delta_x: BigInt;

    if sqrt_price_next > sqrt_price_current {
        // Price is increasing
        let numerator = (liquidity * (sqrt_price_next - sqrt_price_current)) << 64;
        let denominator = sqrt_price_next * sqrt_price_current;

        if round_up {
            delta_x = MathUtil::div_round_up(&numerator, &denominator);
        } else {
            delta_x = numerator / denominator;
        }
    } else {
        // Price is decreasing
        let numerator = (liquidity * (sqrt_price_current - sqrt_price_next)) << 64;
        let denominator = sqrt_price_next * sqrt_price_current;

        if round_up {
            delta_x = MathUtil::div_round_up(&numerator, &denominator);
        } else {
            delta_x = numerator / denominator;
        }
    }

    delta_x
}

/// Compute the amount of token B corresponding to a given change in sqrt price
pub fn compute_amount_b_delta(liquidity: &BigInt, sqrt_price_next: &BigInt, sqrt_price_current: &BigInt, round_up: bool) -> BigInt {
    let delta_y: BigInt;

    if sqrt_price_next > sqrt_price_current {
        // Price is increasing
        let delta = sqrt_price_next - sqrt_price_current;

        if round_up {
            delta_y = MathUtil::div_round_up(&(liquidity * delta), &BigInt::from(2).pow(64));
        } else {
            delta_y = (liquidity * delta) >> 64;
        }
    } else {
        // Price is decreasing
        let delta = sqrt_price_current - sqrt_price_next;

        if round_up {
            delta_y = MathUtil::div_round_up(&(liquidity * delta), &BigInt::from(2).pow(64));
        } else {
            delta_y = (liquidity * delta) >> 64;
        }
    }

    delta_y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_amount_a_delta() {
        // Test case 1: sqrt price increases
        let liquidity = BigInt::from(1000000);
        let sqrt_price_current = BigInt::from(2).pow(64); // 1.0 in x64
        let sqrt_price_next = sqrt_price_current.clone() * BigInt::from(11) / BigInt::from(10); // 1.1 in x64

        // Without rounding up
        let amount_a = compute_amount_a_delta(&liquidity, &sqrt_price_next, &sqrt_price_current, false);

        // With rounding up
        let amount_a_rounded = compute_amount_a_delta(&liquidity, &sqrt_price_next, &sqrt_price_current, true);

        // The amount_a_rounded should be >= amount_a
        assert!(amount_a_rounded >= amount_a);

        // Test case 2: sqrt price decreases
        let sqrt_price_next = sqrt_price_current.clone() * BigInt::from(9) / BigInt::from(10); // 0.9 in x64

        // Without rounding up
        let amount_a = compute_amount_a_delta(&liquidity, &sqrt_price_next, &sqrt_price_current, false);

        // With rounding up
        let amount_a_rounded = compute_amount_a_delta(&liquidity, &sqrt_price_next, &sqrt_price_current, true);

        // The amount_a_rounded should be >= amount_a
        assert!(amount_a_rounded >= amount_a);
    }

    #[test]
    fn test_compute_amount_b_delta() {
        // Test case 1: sqrt price increases
        let liquidity = BigInt::from(1000000);
        let sqrt_price_current = BigInt::from(2).pow(64); // 1.0 in x64
        let sqrt_price_next = sqrt_price_current.clone() * BigInt::from(11) / BigInt::from(10); // 1.1 in x64

        // Without rounding up
        let amount_b = compute_amount_b_delta(&liquidity, &sqrt_price_next, &sqrt_price_current, false);

        // With rounding up
        let amount_b_rounded = compute_amount_b_delta(&liquidity, &sqrt_price_next, &sqrt_price_current, true);

        // The amount_b_rounded should be >= amount_b
        assert!(amount_b_rounded >= amount_b);

        // Test case 2: sqrt price decreases
        let sqrt_price_next = sqrt_price_current.clone() * BigInt::from(9) / BigInt::from(10); // 0.9 in x64

        // Without rounding up
        let amount_b = compute_amount_b_delta(&liquidity, &sqrt_price_next, &sqrt_price_current, false);

        // With rounding up
        let amount_b_rounded = compute_amount_b_delta(&liquidity, &sqrt_price_next, &sqrt_price_current, true);

        // The amount_b_rounded should be >= amount_b
        assert!(amount_b_rounded >= amount_b);
    }

    #[test]
    fn test_compute_swap_step_a_to_b_exact_input() {
        // Test swapping A to B with exact input
        let sqrt_price_current = BigInt::from(2).pow(64); // 1.0 in x64
        let sqrt_price_target = BigInt::from(2).pow(64) * BigInt::from(8) / BigInt::from(10); // 0.8 in x64
        let liquidity = BigInt::from(1000000);
        let amount_remaining = BigInt::from(1000);
        let fee_rate = BigInt::from(3000); // 0.3%
        let amount_specified_is_input = true;
        let a2b = true;

        let result = compute_swap_step(
            &sqrt_price_current,
            &sqrt_price_target,
            &liquidity,
            &amount_remaining,
            &fee_rate,
            amount_specified_is_input,
            a2b,
        );

        // Verify result properties
        assert!(result.next_sqrt_price <= sqrt_price_current);
        assert!(result.next_sqrt_price >= sqrt_price_target);
        assert!(result.amount_a > BigInt::zero());
        assert!(result.amount_b > BigInt::zero());
        assert!(result.fee_amount > BigInt::zero());

        // The input amount minus fees should equal amount_a
        let expected_input_minus_fees = &amount_remaining - &amount_remaining * &fee_rate / BigInt::from(1_000_000);
        assert!(result.amount_a <= expected_input_minus_fees);
    }

    #[test]
    fn test_compute_swap_step_a_to_b_exact_output() {
        // Test swapping A to B with exact output
        let sqrt_price_current = BigInt::from(2).pow(64); // 1.0 in x64
        let sqrt_price_target = BigInt::from(2).pow(64) * BigInt::from(8) / BigInt::from(10); // 0.8 in x64
        let liquidity = BigInt::from(1000000);
        let amount_remaining = BigInt::from(1000); // Exact output amount
        let fee_rate = BigInt::from(3000); // 0.3%
        let amount_specified_is_input = false;
        let a2b = true;

        let result = compute_swap_step(
            &sqrt_price_current,
            &sqrt_price_target,
            &liquidity,
            &amount_remaining,
            &fee_rate,
            amount_specified_is_input,
            a2b,
        );

        // Verify result properties
        assert!(result.next_sqrt_price <= sqrt_price_current);
        assert!(result.next_sqrt_price >= sqrt_price_target);
        assert!(result.amount_a > BigInt::zero());
        assert!(result.amount_b > BigInt::zero());

        // The output amount should be close to amount_remaining
        assert!(result.amount_b <= amount_remaining);
    }

    #[test]
    fn test_compute_swap_step_b_to_a_exact_input() {
        // Test swapping B to A with exact input
        let sqrt_price_current = BigInt::from(2).pow(64); // 1.0 in x64
        let sqrt_price_target = BigInt::from(2).pow(64) * BigInt::from(12) / BigInt::from(10); // 1.2 in x64
        let liquidity = BigInt::from(1000000);
        let amount_remaining = BigInt::from(1000);
        let fee_rate = BigInt::from(3000); // 0.3%
        let amount_specified_is_input = true;
        let a2b = false;

        let result = compute_swap_step(
            &sqrt_price_current,
            &sqrt_price_target,
            &liquidity,
            &amount_remaining,
            &fee_rate,
            amount_specified_is_input,
            a2b,
        );

        // Verify result properties
        assert!(result.next_sqrt_price >= sqrt_price_current);
        assert!(result.next_sqrt_price <= sqrt_price_target);
        assert!(result.amount_a > BigInt::zero());
        assert!(result.amount_b > BigInt::zero());
        assert!(result.fee_amount > BigInt::zero());

        // The input amount minus fees should equal amount_b
        let expected_input_minus_fees = &amount_remaining - &amount_remaining * &fee_rate / BigInt::from(1_000_000);
        assert!(result.amount_b <= expected_input_minus_fees);
    }

    #[test]
    fn test_compute_swap_step_b_to_a_exact_output() {
        // Test swapping B to A with exact output
        let sqrt_price_current = BigInt::from(2).pow(64); // 1.0 in x64
        let sqrt_price_target = BigInt::from(2).pow(64) * BigInt::from(12) / BigInt::from(10); // 1.2 in x64
        let liquidity = BigInt::from(1000000);
        let amount_remaining = BigInt::from(1000); // Exact output amount
        let fee_rate = BigInt::from(3000); // 0.3%
        let amount_specified_is_input = false;
        let a2b = false;

        let result = compute_swap_step(
            &sqrt_price_current,
            &sqrt_price_target,
            &liquidity,
            &amount_remaining,
            &fee_rate,
            amount_specified_is_input,
            a2b,
        );

        // Verify result properties
        assert!(result.next_sqrt_price >= sqrt_price_current);
        assert!(result.next_sqrt_price <= sqrt_price_target);
        assert!(result.amount_a > BigInt::zero());
        assert!(result.amount_b > BigInt::zero());

        // The output amount should be close to amount_remaining
        assert!(result.amount_a <= amount_remaining);
    }
}
