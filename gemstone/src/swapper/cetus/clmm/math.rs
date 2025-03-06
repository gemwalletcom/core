use super::constants::*;
use super::error::ErrorCode;
use bigdecimal::BigDecimal;
use num_bigint::{BigInt, Sign};
use num_traits::{One, Signed};
use std::str::FromStr;

/// Convert a number to X64 format (BN variant)
pub fn to_x64_bn(num: &BigInt) -> BigInt {
    num * (BigInt::from(2).pow(64))
}

/// Convert a number to X64 format (Decimal variant)
pub fn to_x64_decimal(num: &BigDecimal) -> BigDecimal {
    num * BigDecimal::from_str("18446744073709551616").unwrap() // 2^64
}

/// Convert a decimal to X64 BigInt
pub fn to_x64(num: &BigDecimal) -> BigInt {
    let scaled = num * BigDecimal::from_str("18446744073709551616").unwrap(); // 2^64
    BigInt::from_str(&scaled.with_prec(0).to_string()).unwrap_or_else(|_| BigInt::from(0))
}

/// Convert from X64 BigInt to Decimal
pub fn from_x64(num: &BigInt) -> BigDecimal {
    let num_decimal = BigDecimal::from_str(&num.to_string()).unwrap();
    num_decimal / BigDecimal::from_str("18446744073709551616").unwrap() // 2^64
}

/// Convert from X64 Decimal to Decimal
pub fn from_x64_decimal(num: &BigDecimal) -> BigDecimal {
    num / BigDecimal::from_str("18446744073709551616").unwrap() // 2^64
}

/// Convert from X64 BigInt to BigInt
pub fn from_x64_bn(num: &BigInt) -> BigInt {
    num / (BigInt::from(2).pow(64))
}

/// Shift right with round up if needed
pub fn shift_right_round_up(n: &BigInt) -> BigInt {
    let mut result = n >> 64;

    if n % &*U64_MAX > *ZERO {
        result += BigInt::one();
    }

    result
}

/// Division with round up
pub fn div_round_up(n0: &BigInt, n1: &BigInt) -> BigInt {
    let has_remainder = n0 % n1 != *ZERO;
    if has_remainder {
        n0 / n1 + &*ONE
    } else {
        n0 / n1
    }
}

/// Subtraction with underflow handling for u128
pub fn sub_underflow_u128(n0: &BigInt, n1: &BigInt) -> BigInt {
    if n0 < n1 {
        n0 - n1 + &*U128_MAX
    } else {
        n0 - n1
    }
}

/// Check unsigned subtraction
pub fn check_unsigned_sub(n0: &BigInt, n1: &BigInt) -> Result<BigInt, ErrorCode> {
    let n = n0 - n1;
    if n.sign() == Sign::Minus {
        Err(ErrorCode::UnsignedIntegerOverflow)
    } else {
        Ok(n)
    }
}

/// Check multiplication with overflow checking
pub fn check_mul(n0: &BigInt, n1: &BigInt, limit: u32) -> Result<BigInt, ErrorCode> {
    let n = n0 * n1;
    if is_overflow(&n, limit) {
        Err(ErrorCode::MulOverflow)
    } else {
        Ok(n)
    }
}

/// Check multiplication and division with floor rounding
pub fn check_mul_div_floor(n0: &BigInt, n1: &BigInt, denom: &BigInt, limit: u32) -> Result<BigInt, ErrorCode> {
    if denom.eq(&*ZERO) {
        return Err(ErrorCode::DivideByZero);
    }

    let n = (n0 * n1) / denom;

    if is_overflow(&n, limit) {
        Err(ErrorCode::MulDivOverflow)
    } else {
        Ok(n)
    }
}

/// Check multiplication and division with ceiling rounding
pub fn check_mul_div_ceil(n0: &BigInt, n1: &BigInt, denom: &BigInt, limit: u32) -> Result<BigInt, ErrorCode> {
    if denom.eq(&*ZERO) {
        return Err(ErrorCode::DivideByZero);
    }

    let n = (n0 * n1 + (denom - 1)) / denom;

    if is_overflow(&n, limit) {
        Err(ErrorCode::MulDivOverflow)
    } else {
        Ok(n)
    }
}

/// Check multiplication and division with rounding
pub fn check_mul_div_round(n0: &BigInt, n1: &BigInt, denom: &BigInt, limit: u32) -> Result<BigInt, ErrorCode> {
    if denom.eq(&*ZERO) {
        return Err(ErrorCode::DivideByZero);
    }

    let n = (n0 * (n1 + (denom >> 1))) / denom;

    if is_overflow(&n, limit) {
        Err(ErrorCode::MulDivOverflow)
    } else {
        Ok(n)
    }
}

/// Check multiplication and shift right
pub fn check_mul_shift_right(n0: &BigInt, n1: &BigInt, shift: u32, limit: u32) -> Result<BigInt, ErrorCode> {
    let n = (n0 * n1) / (BigInt::from(2).pow(shift));

    if is_overflow(&n, limit) {
        Err(ErrorCode::MulShiftRightOverflow)
    } else {
        Ok(n)
    }
}

/// Check multiplication and shift right 64 with conditional round up
pub fn check_mul_shift_right64_round_up_if(n0: &BigInt, n1: &BigInt, limit: u32, round_up: bool) -> Result<BigInt, ErrorCode> {
    let p = n0 * n1;
    let should_round_up = round_up && (&p & &*U64_MAX) > *ZERO;
    let result = if should_round_up { &p >> (64 + 1) } else { &p >> 64 };

    if is_overflow(&result, limit) {
        Err(ErrorCode::MulShiftRightOverflow)
    } else {
        Ok(result)
    }
}

/// Check multiplication and shift left
pub fn check_mul_shift_left(n0: &BigInt, n1: &BigInt, shift: u32, limit: u32) -> Result<BigInt, ErrorCode> {
    let n = (n0 * n1) << shift;

    if is_overflow(&n, limit) {
        Err(ErrorCode::MulShiftLeftOverflow)
    } else {
        Ok(n)
    }
}

/// Check division with conditional round up
pub fn check_div_round_up_if(n0: &BigInt, n1: &BigInt, round_up: bool) -> Result<BigInt, ErrorCode> {
    if n1.eq(&*ZERO) {
        return Err(ErrorCode::DivideByZero);
    }

    if round_up {
        Ok(div_round_up(n0, n1))
    } else {
        Ok(n0 / n1)
    }
}

/// Check if a number overflows a specific bit width
pub fn is_overflow(n: &BigInt, bit: u32) -> bool {
    n >= &(BigInt::from(2).pow(bit))
}

/// Get the sign of a BigInt (0 for positive, 1 for negative)
pub fn sign(v: &BigInt) -> usize {
    if v.bit(127) {
        1
    } else {
        0
    }
}

/// Check if a BigInt is negative
pub fn is_neg(v: &BigInt) -> bool {
    sign(v) == 1
}

/// Get the absolute value of a u128
pub fn abs_u128(v: &BigInt) -> BigInt {
    if v > &*ZERO {
        v.clone()
    } else {
        u128_neg(&(v - 1))
    }
}

/// Negate a u128 value
pub fn u128_neg(v: &BigInt) -> BigInt {
    v ^ BigInt::from_str("ffffffffffffffffffffffffffffffff").unwrap()
}

/// Get the negative of a BigInt
pub fn neg(v: &BigInt) -> BigInt {
    if is_neg(v) {
        v.abs()
    } else {
        neg_from(v)
    }
}

/// Get the absolute value of a BigInt
pub fn abs(v: &BigInt) -> BigInt {
    if sign(v) == 0 {
        v.clone()
    } else {
        u128_neg(&(v - 1))
    }
}

/// Get negative from a positive BigInt
pub fn neg_from(v: &BigInt) -> BigInt {
    if v == &*ZERO {
        v.clone()
    } else {
        (u128_neg(v) + 1) | (BigInt::from(1) << 127)
    }
}

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
    div_round_up(&numerator, &denominator)
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
    div_round_up(&numerator, &denominator)
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
    sqrt_price_x64 - div_round_up(&(amount << 64), liquidity)
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

    #[test]
    fn test_to_x64_bn() {
        let num = BigInt::from(123);
        let x64 = to_x64_bn(&num);

        assert_eq!(x64, num * (BigInt::from(2).pow(64)));
    }

    #[test]
    fn test_from_x64_bn() {
        let original = BigInt::from(123);
        let x64 = to_x64_bn(&original);
        let round_trip = from_x64_bn(&x64);

        assert_eq!(original, round_trip);
    }

    #[test]
    fn test_div_round_up() {
        // Test with no remainder
        let n0 = BigInt::from(10);
        let n1 = BigInt::from(2);
        let result = div_round_up(&n0, &n1);
        assert_eq!(result, BigInt::from(5));

        // Test with remainder
        let n0 = BigInt::from(10);
        let n1 = BigInt::from(3);
        let result = div_round_up(&n0, &n1);
        assert_eq!(result, BigInt::from(4));
    }

    #[test]
    fn test_sub_underflow_u128() {
        // No underflow case
        let n0 = BigInt::from(10);
        let n1 = BigInt::from(5);
        let result = sub_underflow_u128(&n0, &n1);
        assert_eq!(result, BigInt::from(5));

        // Underflow case
        let n0 = BigInt::from(5);
        let n1 = BigInt::from(10);
        let result = sub_underflow_u128(&n0, &n1);
        assert_eq!(result, &n0 - &n1 + &*U128_MAX);
    }

    #[test]
    fn test_check_unsigned_sub() {
        // No underflow case
        let n0 = BigInt::from(10);
        let n1 = BigInt::from(5);
        let result = check_unsigned_sub(&n0, &n1).unwrap();
        assert_eq!(result, BigInt::from(5));

        // Underflow case
        let n0 = BigInt::from(5);
        let n1 = BigInt::from(10);
        let result = check_unsigned_sub(&n0, &n1);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_overflow() {
        // No overflow case
        let n = BigInt::from(15); // 1111 in binary
        let bit_width = 4;
        let result = is_overflow(&n, bit_width);
        assert!(!result);

        // No overflow case (edge case)
        let n = BigInt::from(15); // 1111 in binary
        let bit_width = 5;
        let result = is_overflow(&n, bit_width);
        assert!(!result);

        // Overflow case
        let n = BigInt::from(16); // 10000 in binary
        let bit_width = 4;
        let result = is_overflow(&n, bit_width);
        assert!(result);
    }

    #[test]
    fn test_sign_and_is_neg() {
        // Positive number
        let v = BigInt::from(123);
        assert_eq!(sign(&v), 0);
        assert!(!is_neg(&v));

        // Create a negative number in u128 representation
        let mut v = BigInt::from(0);
        v.set_bit(127, true);
        assert_eq!(sign(&v), 1);
        assert!(is_neg(&v));
    }

    #[test]
    fn test_get_default_sqrt_price_limit() {
        let a2b_limit = get_default_sqrt_price_limit(true);
        let b2a_limit = get_default_sqrt_price_limit(false);

        assert_eq!(a2b_limit, *MIN_SQRT_PRICE);
        assert_eq!(b2a_limit, *MAX_SQRT_PRICE);
    }

    #[test]
    fn test_get_default_other_amount_threshold() {
        let input_threshold = get_default_other_amount_threshold(true);
        let output_threshold = get_default_other_amount_threshold(false);

        assert_eq!(input_threshold, *ZERO);
        assert_eq!(output_threshold, *U64_MAX);
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
