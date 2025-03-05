use super::constants::*;
use super::error::{ClmmpoolsError, MathErrorCode};
use bigdecimal::BigDecimal;
use num_bigint::{BigInt, Sign};
use num_traits::{One, Signed};
use std::str::FromStr;

/// Math utilities for the CLMM pool
pub struct MathUtil;

impl MathUtil {
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
        BigInt::from_str(&scaled.to_string()).unwrap()
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
    pub fn check_unsigned_sub(n0: &BigInt, n1: &BigInt) -> Result<BigInt, ClmmpoolsError> {
        let n = n0 - n1;
        if n.sign() == Sign::Minus {
            Err(ClmmpoolsError::math_error(
                "Unsigned integer sub overflow",
                MathErrorCode::UnsignedIntegerOverflow,
            ))
        } else {
            Ok(n)
        }
    }

    /// Check multiplication with overflow checking
    pub fn check_mul(n0: &BigInt, n1: &BigInt, limit: u32) -> Result<BigInt, ClmmpoolsError> {
        let n = n0 * n1;
        if Self::is_overflow(&n, limit) {
            Err(ClmmpoolsError::math_error("Multiplication overflow", MathErrorCode::MulOverflow))
        } else {
            Ok(n)
        }
    }

    /// Check multiplication and division with floor rounding
    pub fn check_mul_div_floor(n0: &BigInt, n1: &BigInt, denom: &BigInt, limit: u32) -> Result<BigInt, ClmmpoolsError> {
        if denom.eq(&*ZERO) {
            return Err(ClmmpoolsError::math_error("Divide by zero", MathErrorCode::DivideByZero));
        }

        let n = (n0 * n1) / denom;

        if Self::is_overflow(&n, limit) {
            Err(ClmmpoolsError::math_error("Multiplication div overflow", MathErrorCode::MulDivOverflow))
        } else {
            Ok(n)
        }
    }

    /// Check multiplication and division with ceiling rounding
    pub fn check_mul_div_ceil(n0: &BigInt, n1: &BigInt, denom: &BigInt, limit: u32) -> Result<BigInt, ClmmpoolsError> {
        if denom.eq(&*ZERO) {
            return Err(ClmmpoolsError::math_error("Divide by zero", MathErrorCode::DivideByZero));
        }

        let n = (n0 * n1 + (denom - 1)) / denom;

        if Self::is_overflow(&n, limit) {
            Err(ClmmpoolsError::math_error("Multiplication div overflow", MathErrorCode::MulDivOverflow))
        } else {
            Ok(n)
        }
    }

    /// Check multiplication and division with rounding
    pub fn check_mul_div_round(n0: &BigInt, n1: &BigInt, denom: &BigInt, limit: u32) -> Result<BigInt, ClmmpoolsError> {
        if denom.eq(&*ZERO) {
            return Err(ClmmpoolsError::math_error("Divide by zero", MathErrorCode::DivideByZero));
        }

        let n = (n0 * (n1 + (denom >> 1))) / denom;

        if Self::is_overflow(&n, limit) {
            Err(ClmmpoolsError::math_error("Multiplication div overflow", MathErrorCode::MulDivOverflow))
        } else {
            Ok(n)
        }
    }

    /// Check multiplication and shift right
    pub fn check_mul_shift_right(n0: &BigInt, n1: &BigInt, shift: u32, limit: u32) -> Result<BigInt, ClmmpoolsError> {
        let n = (n0 * n1) / (BigInt::from(2).pow(shift));

        if Self::is_overflow(&n, limit) {
            Err(ClmmpoolsError::math_error(
                "Multiplication shift right overflow",
                MathErrorCode::MulShiftRightOverflow,
            ))
        } else {
            Ok(n)
        }
    }

    /// Check multiplication and shift right 64 with conditional round up
    pub fn check_mul_shift_right64_round_up_if(n0: &BigInt, n1: &BigInt, limit: u32, round_up: bool) -> Result<BigInt, ClmmpoolsError> {
        let p = n0 * n1;
        let should_round_up = round_up && (&p & &*U64_MAX) > *ZERO;
        let result = if should_round_up { &p >> (64 + 1) } else { &p >> 64 };

        if Self::is_overflow(&result, limit) {
            Err(ClmmpoolsError::math_error(
                "Multiplication shift right overflow",
                MathErrorCode::MulShiftRightOverflow,
            ))
        } else {
            Ok(result)
        }
    }

    /// Check multiplication and shift left
    pub fn check_mul_shift_left(n0: &BigInt, n1: &BigInt, shift: u32, limit: u32) -> Result<BigInt, ClmmpoolsError> {
        let n = (n0 * n1) << shift;

        if Self::is_overflow(&n, limit) {
            Err(ClmmpoolsError::math_error(
                "Multiplication shift left overflow",
                MathErrorCode::MulShiftLeftOverflow,
            ))
        } else {
            Ok(n)
        }
    }

    /// Check division with conditional round up
    pub fn check_div_round_up_if(n0: &BigInt, n1: &BigInt, round_up: bool) -> Result<BigInt, ClmmpoolsError> {
        if n1.eq(&*ZERO) {
            return Err(ClmmpoolsError::math_error("Divide by zero", MathErrorCode::DivideByZero));
        }

        if round_up {
            Ok(Self::div_round_up(n0, n1))
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
        Self::sign(v) == 1
    }

    /// Get the absolute value of a u128
    pub fn abs_u128(v: &BigInt) -> BigInt {
        if v > &*ZERO {
            v.clone()
        } else {
            Self::u128_neg(&(v - 1))
        }
    }

    /// Negate a u128 value
    pub fn u128_neg(v: &BigInt) -> BigInt {
        v ^ BigInt::from_str("ffffffffffffffffffffffffffffffff").unwrap()
    }

    /// Get the negative of a BigInt
    pub fn neg(v: &BigInt) -> BigInt {
        if Self::is_neg(v) {
            v.abs()
        } else {
            Self::neg_from(v)
        }
    }

    /// Get the absolute value of a BigInt
    pub fn abs(v: &BigInt) -> BigInt {
        if Self::sign(v) == 0 {
            v.clone()
        } else {
            Self::u128_neg(&(v - 1))
        }
    }

    /// Get negative from a positive BigInt
    pub fn neg_from(v: &BigInt) -> BigInt {
        if v == &*ZERO {
            v.clone()
        } else {
            (Self::u128_neg(v) + 1) | (BigInt::from(1) << 127)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_x64_bn() {
        let num = BigInt::from(123);
        let x64 = MathUtil::to_x64_bn(&num);

        assert_eq!(x64, num * (BigInt::from(2).pow(64)));
    }

    #[test]
    fn test_from_x64_bn() {
        let original = BigInt::from(123);
        let x64 = MathUtil::to_x64_bn(&original);
        let round_trip = MathUtil::from_x64_bn(&x64);

        assert_eq!(original, round_trip);
    }

    #[test]
    fn test_div_round_up() {
        // Test with no remainder
        let n0 = BigInt::from(10);
        let n1 = BigInt::from(2);
        let result = MathUtil::div_round_up(&n0, &n1);
        assert_eq!(result, BigInt::from(5));

        // Test with remainder
        let n0 = BigInt::from(10);
        let n1 = BigInt::from(3);
        let result = MathUtil::div_round_up(&n0, &n1);
        assert_eq!(result, BigInt::from(4));
    }

    #[test]
    fn test_sub_underflow_u128() {
        // No underflow case
        let n0 = BigInt::from(10);
        let n1 = BigInt::from(5);
        let result = MathUtil::sub_underflow_u128(&n0, &n1);
        assert_eq!(result, BigInt::from(5));

        // Underflow case
        let n0 = BigInt::from(5);
        let n1 = BigInt::from(10);
        let result = MathUtil::sub_underflow_u128(&n0, &n1);
        assert_eq!(result, &n0 - &n1 + &*U128_MAX);
    }

    #[test]
    fn test_check_unsigned_sub() {
        // No underflow case
        let n0 = BigInt::from(10);
        let n1 = BigInt::from(5);
        let result = MathUtil::check_unsigned_sub(&n0, &n1).unwrap();
        assert_eq!(result, BigInt::from(5));

        // Underflow case
        let n0 = BigInt::from(5);
        let n1 = BigInt::from(10);
        let result = MathUtil::check_unsigned_sub(&n0, &n1);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_overflow() {
        // No overflow case
        let n = BigInt::from(15); // 1111 in binary
        let bit_width = 4;
        let result = MathUtil::is_overflow(&n, bit_width);
        assert!(!result);

        // No overflow case (edge case)
        let n = BigInt::from(15); // 1111 in binary
        let bit_width = 5;
        let result = MathUtil::is_overflow(&n, bit_width);
        assert!(!result);

        // Overflow case
        let n = BigInt::from(16); // 10000 in binary
        let bit_width = 4;
        let result = MathUtil::is_overflow(&n, bit_width);
        assert!(result);
    }

    #[test]
    fn test_sign_and_is_neg() {
        // Positive number
        let v = BigInt::from(123);
        assert_eq!(MathUtil::sign(&v), 0);
        assert!(!MathUtil::is_neg(&v));

        // Create a negative number in u128 representation
        let mut v = BigInt::from(0);
        v.set_bit(127, true);
        assert_eq!(MathUtil::sign(&v), 1);
        assert!(MathUtil::is_neg(&v));
    }
}
