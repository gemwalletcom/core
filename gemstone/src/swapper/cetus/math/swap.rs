use super::constants::{MAX_SQRT_PRICE, MIN_SQRT_PRICE, U64_MAX, ZERO};
use num_bigint::BigInt;

pub struct SwapUtils;

impl SwapUtils {
    /// Get the default sqrt price limit for a swap.
    ///
    /// # Arguments
    ///
    /// * `a2b` - true if the swap is A to B, false if the swap is B to A.
    ///
    /// # Returns
    ///
    /// * The default sqrt price limit for the swap.
    pub fn get_default_sqrt_price_limit(a2b: bool) -> BigInt {
        if a2b {
            MIN_SQRT_PRICE.clone()
        } else {
            MAX_SQRT_PRICE.clone()
        }
    }

    /// Get the default values for the otherAmountThreshold in a swap.
    ///
    /// # Arguments
    ///
    /// * `amount_specified_is_input` - The direction of a swap
    ///
    /// # Returns
    ///
    /// * The default values for the otherAmountThreshold parameter in a swap.
    pub fn get_default_other_amount_threshold(amount_specified_is_input: bool) -> BigInt {
        if amount_specified_is_input {
            ZERO.clone()
        } else {
            U64_MAX.clone()
        }
    }
}

/// Math utilities for division with rounding up
pub struct MathUtil;

impl MathUtil {
    pub fn div_round_up(n0: &BigInt, n1: &BigInt) -> BigInt {
        let has_remainder = n0 % n1 != *ZERO;
        if has_remainder {
            n0 / n1 + 1
        } else {
            n0 / n1
        }
    }
}

/// Get lower sqrt price from token A.
///
/// # Arguments
///
/// * `amount` - The amount of tokens the user wanted to swap from.
/// * `liquidity` - The liquidity of the pool.
/// * `sqrt_price_x64` - The sqrt price of the pool.
///
/// # Returns
///
/// * Lower sqrt price X64
pub fn get_lower_sqrt_price_from_coin_a(amount: &BigInt, liquidity: &BigInt, sqrt_price_x64: &BigInt) -> BigInt {
    let numerator = (liquidity * sqrt_price_x64) << 64;
    let denominator = (liquidity << 64) + (amount * sqrt_price_x64);

    // always round up
    MathUtil::div_round_up(&numerator, &denominator)
}

/// Get upper sqrt price from token A.
///
/// # Arguments
///
/// * `amount` - The amount of tokens the user wanted to swap from.
/// * `liquidity` - The liquidity of the pool.
/// * `sqrt_price_x64` - The sqrt price of the pool.
///
/// # Returns
///
/// * Upper sqrt price X64
pub fn get_upper_sqrt_price_from_coin_a(amount: &BigInt, liquidity: &BigInt, sqrt_price_x64: &BigInt) -> BigInt {
    let numerator = (liquidity * sqrt_price_x64) << 64;
    let denominator = (liquidity << 64) - (amount * sqrt_price_x64);

    // always round up
    MathUtil::div_round_up(&numerator, &denominator)
}

/// Get lower sqrt price from coin B.
///
/// # Arguments
///
/// * `amount` - The amount of coins the user wanted to swap from.
/// * `liquidity` - The liquidity of the pool.
/// * `sqrt_price_x64` - The sqrt price of the pool.
///
/// # Returns
///
/// * Lower sqrt price X64
pub fn get_lower_sqrt_price_from_coin_b(amount: &BigInt, liquidity: &BigInt, sqrt_price_x64: &BigInt) -> BigInt {
    // always round down (rounding up a negative number)
    sqrt_price_x64 - MathUtil::div_round_up(&(amount << 64), liquidity)
}

/// Get upper sqrt price from coin B.
///
/// # Arguments
///
/// * `amount` - The amount of coins the user wanted to swap from.
/// * `liquidity` - The liquidity of the pool.
/// * `sqrt_price_x64` - The sqrt price of the pool.
///
/// # Returns
///
/// * Upper sqrt price X64
pub fn get_upper_sqrt_price_from_coin_b(amount: &BigInt, liquidity: &BigInt, sqrt_price_x64: &BigInt) -> BigInt {
    // always round down (rounding up a negative number)
    sqrt_price_x64 + ((amount << 64) / liquidity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    #[test]
    fn test_get_default_sqrt_price_limit() {
        let a2b_limit = SwapUtils::get_default_sqrt_price_limit(true);
        let b2a_limit = SwapUtils::get_default_sqrt_price_limit(false);

        assert_eq!(a2b_limit, *MIN_SQRT_PRICE);
        assert_eq!(b2a_limit, *MAX_SQRT_PRICE);
    }

    #[test]
    fn test_get_default_other_amount_threshold() {
        let input_threshold = SwapUtils::get_default_other_amount_threshold(true);
        let output_threshold = SwapUtils::get_default_other_amount_threshold(false);

        assert_eq!(input_threshold, *ZERO);
        assert_eq!(output_threshold, *U64_MAX);
    }

    #[test]
    fn test_div_round_up() {
        let n0 = BigInt::from(10);
        let n1 = BigInt::from(3);
        let result = MathUtil::div_round_up(&n0, &n1);

        assert_eq!(result, BigInt::from(4));

        // Test with exact division
        let n0 = BigInt::from(10);
        let n1 = BigInt::from(5);
        let result = MathUtil::div_round_up(&n0, &n1);

        assert_eq!(result, BigInt::from(2));
    }

    #[test]
    fn test_get_lower_sqrt_price_from_coin_a() {
        // Example values
        let amount = BigInt::from(1000);
        let liquidity = BigInt::from(5000);
        let sqrt_price_x64 = BigInt::from(2).pow(64); // 1.0 in x64 representation

        let result = get_lower_sqrt_price_from_coin_a(&amount, &liquidity, &sqrt_price_x64);

        // Verify the result is correct
        // For given values and using the formula from the function:
        // numerator = 5000 * 2^64 * 2^64
        // denominator = 5000 * 2^64 + 1000 * 2^64 = (5000 + 1000) * 2^64 = 6000 * 2^64
        // expected = numerator / denominator = 5000/6000 = 5/6
        // In x64, this should be approximately 2^64 * (5/6)
        let expected = BigInt::from(2).pow(64) * BigInt::from(5) / BigInt::from(6);

        // Allow for small rounding differences
        let diff = if result > expected { &result - &expected } else { &expected - &result };

        assert!(diff <= BigInt::from(1), "Difference too large: {}", diff);
    }

    #[test]
    fn test_get_upper_sqrt_price_from_coin_a() {
        // Example values
        let amount = BigInt::from(1000);
        let liquidity = BigInt::from(5000);
        let sqrt_price_x64 = BigInt::from(2).pow(64); // 1.0 in x64 representation

        let result = get_upper_sqrt_price_from_coin_a(&amount, &liquidity, &sqrt_price_x64);

        // Verify the result is correct
        // For given values and using the formula from the function:
        // numerator = 5000 * 2^64 * 2^64
        // denominator = 5000 * 2^64 - 1000 * 2^64 = (5000 - 1000) * 2^64 = 4000 * 2^64
        // expected = numerator / denominator = 5000/4000 = 5/4 = 1.25
        // In x64, this should be approximately 2^64 * (5/4)
        let expected = BigInt::from(2).pow(64) * BigInt::from(5) / BigInt::from(4);

        // Allow for small rounding differences
        let diff = if result > expected { &result - &expected } else { &expected - &result };

        assert!(diff <= BigInt::from(1), "Difference too large: {}", diff);
    }

    #[test]
    fn test_get_lower_sqrt_price_from_coin_b() {
        // Example values
        let amount = BigInt::from(1000);
        let liquidity = BigInt::from(5000);
        let sqrt_price_x64 = BigInt::from(2).pow(64); // 1.0 in x64 representation

        let result = get_lower_sqrt_price_from_coin_b(&amount, &liquidity, &sqrt_price_x64);

        // For given values and using the formula from the function:
        // sqrt_price_x64 - (amount << 64) / liquidity with rounding
        // = 2^64 - (1000 * 2^64) / 5000 with rounding
        // = 2^64 - 2^64 * (1000/5000) = 2^64 * (1 - 1/5) = 2^64 * (4/5)
        let expected = BigInt::from(2).pow(64) * BigInt::from(4) / BigInt::from(5);

        // Allow for small rounding differences
        let diff = if result > expected { &result - &expected } else { &expected - &result };

        assert!(diff <= BigInt::from(1), "Difference too large: {}", diff);
    }

    #[test]
    fn test_get_upper_sqrt_price_from_coin_b() {
        // Example values
        let amount = BigInt::from(1000);
        let liquidity = BigInt::from(5000);
        let sqrt_price_x64 = BigInt::from(2).pow(64); // 1.0 in x64 representation

        let result = get_upper_sqrt_price_from_coin_b(&amount, &liquidity, &sqrt_price_x64);

        // For given values and using the formula from the function:
        // sqrt_price_x64 + (amount << 64) / liquidity
        // = 2^64 + (1000 * 2^64) / 5000
        // = 2^64 + 2^64 * (1000/5000) = 2^64 * (1 + 1/5) = 2^64 * (6/5)
        let expected = BigInt::from(2).pow(64) * BigInt::from(6) / BigInt::from(5);

        // Allow for small rounding differences
        let diff = if result > expected { &result - &expected } else { &expected - &result };

        assert!(diff <= BigInt::from(1), "Difference too large: {}", diff);
    }
}
