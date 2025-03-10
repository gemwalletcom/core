use super::constants::*;
use super::error::ErrorCode;
use num_bigint::{BigInt, Sign};

/// Division with round up
pub fn div_round_up(n0: &BigInt, n1: &BigInt) -> BigInt {
    let has_remainder = n0 % n1 != *ZERO;
    if has_remainder {
        n0 / n1 + &*ONE
    } else {
        n0 / n1
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
