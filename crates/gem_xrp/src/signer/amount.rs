use std::str::FromStr;

use bigdecimal::{BigDecimal, Signed, Zero};
use num_traits::ToPrimitive;
use primitives::SignerError;

use crate::address::XrpAddress;

const POS_SIGN_BIT_MASK: u64 = 0x4000_0000_0000_0000;
const ISSUED_CURRENCY_ZERO: u64 = 0x8000_0000_0000_0000;
const MIN_MANTISSA: u128 = 1_000_000_000_000_000;
const MAX_MANTISSA: u128 = 9_999_999_999_999_999;
const MIN_IOU_EXPONENT: i32 = -96;
const MAX_IOU_EXPONENT: i32 = 80;
const MAX_IOU_PRECISION: usize = 16;
const MAX_DROPS: u64 = 100_000_000_000_000_000;
const CURRENCY_CODE_LENGTH: usize = 20;
const STANDARD_CURRENCY_CODE_LENGTH: usize = 3;
const STANDARD_CURRENCY_CODE_OFFSET: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum XrpAmount {
    Native(u64),
    Issued(IssuedAmount),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IssuedAmount {
    value: BigDecimal,
    currency: [u8; CURRENCY_CODE_LENGTH],
    issuer: XrpAddress,
}

impl XrpAmount {
    pub(crate) fn native(value: &str) -> Result<Self, SignerError> {
        let amount = value.parse::<u64>().map_err(|_| SignerError::invalid_input("invalid XRP amount"))?;
        if amount > MAX_DROPS {
            return Err(SignerError::invalid_input("XRP amount is too large"));
        }
        Ok(Self::Native(amount))
    }

    pub(crate) fn issued(value: &str, currency: &str, issuer: &str) -> Result<Self, SignerError> {
        IssuedAmount::new(value, currency, issuer).map(Self::Issued)
    }

    pub(crate) fn encode(&self, buffer: &mut Vec<u8>) -> Result<(), SignerError> {
        match self {
            Self::Native(amount) => buffer.extend_from_slice(&(*amount | POS_SIGN_BIT_MASK).to_be_bytes()),
            Self::Issued(amount) => amount.encode(buffer)?,
        }
        Ok(())
    }
}

impl IssuedAmount {
    fn new(value: &str, currency: &str, issuer: &str) -> Result<Self, SignerError> {
        Ok(Self {
            value: BigDecimal::from_str(value).map_err(|_| SignerError::invalid_input("invalid XRP token amount"))?,
            currency: currency_code_bytes(currency)?,
            issuer: XrpAddress::parse(issuer)?,
        })
    }

    fn encode(&self, buffer: &mut Vec<u8>) -> Result<(), SignerError> {
        buffer.extend_from_slice(&self.encoded_value()?.to_be_bytes());
        buffer.extend_from_slice(&self.currency);
        buffer.extend_from_slice(self.issuer.as_bytes());
        Ok(())
    }

    fn encoded_value(&self) -> Result<u64, SignerError> {
        validate_issued_value(&self.value)?;

        if self.value.is_zero() {
            return Ok(ISSUED_CURRENCY_ZERO);
        }

        let value = self.value.normalized();
        let is_positive = value.is_positive();
        let (mantissa, scale) = value.as_bigint_and_exponent();
        let mut exponent = -(scale as i32);
        let mut mantissa = mantissa.abs().to_u128().ok_or_else(|| SignerError::invalid_input("invalid XRP token amount"))?;

        while mantissa < MIN_MANTISSA && exponent > MIN_IOU_EXPONENT {
            mantissa *= 10;
            exponent -= 1;
        }

        while mantissa > MAX_MANTISSA {
            if exponent >= MAX_IOU_EXPONENT {
                return Err(SignerError::invalid_input("XRP token amount is too large"));
            }
            mantissa /= 10;
            exponent += 1;
        }

        if exponent < MIN_IOU_EXPONENT || mantissa < MIN_MANTISSA {
            return Ok(ISSUED_CURRENCY_ZERO);
        }

        if exponent > MAX_IOU_EXPONENT || mantissa > MAX_MANTISSA {
            return Err(SignerError::invalid_input("XRP token amount is too large"));
        }

        let mut encoded = ISSUED_CURRENCY_ZERO;
        if is_positive {
            encoded |= POS_SIGN_BIT_MASK;
        }
        encoded |= ((exponent as i64 + 97) as u64) << 54;
        encoded |= mantissa as u64;
        Ok(encoded)
    }
}

fn currency_code_bytes(value: &str) -> Result<[u8; CURRENCY_CODE_LENGTH], SignerError> {
    if value.len() == CURRENCY_CODE_LENGTH * 2 {
        let bytes = hex::decode(value).map_err(|_| SignerError::invalid_input("invalid XRP currency code"))?;
        return bytes.try_into().map_err(|_| SignerError::invalid_input("invalid XRP currency code length"));
    }

    let mut bytes = [0; CURRENCY_CODE_LENGTH];
    let value = value.as_bytes();
    if value.len() > CURRENCY_CODE_LENGTH {
        return Err(SignerError::invalid_input("XRP currency symbol is too long"));
    }
    if value.len() == STANDARD_CURRENCY_CODE_LENGTH {
        let end = STANDARD_CURRENCY_CODE_OFFSET + STANDARD_CURRENCY_CODE_LENGTH;
        bytes[STANDARD_CURRENCY_CODE_OFFSET..end].copy_from_slice(value);
        return Ok(bytes);
    }
    bytes[..value.len()].copy_from_slice(value);
    Ok(bytes)
}

fn validate_issued_value(value: &BigDecimal) -> Result<(), SignerError> {
    if value.is_zero() {
        return Ok(());
    }

    let precision = issued_amount_precision(value);
    if precision > MAX_IOU_PRECISION {
        return Err(SignerError::invalid_input("XRP token amount precision is too large"));
    }

    let exponent = issued_amount_exponent(value);
    if !(MIN_IOU_EXPONENT..=MAX_IOU_EXPONENT).contains(&exponent) {
        return Err(SignerError::invalid_input("XRP token amount exponent is out of range"));
    }

    Ok(())
}

fn issued_amount_precision(value: &BigDecimal) -> usize {
    let (value, _) = value.normalized().into_bigint_and_exponent();
    let (_, digits) = value.into_parts();
    digits.to_string().len()
}

fn issued_amount_exponent(value: &BigDecimal) -> i32 {
    let (_, scale) = value.normalized().as_bigint_and_exponent();
    -(scale as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_bytes() {
        assert_eq!(hex::encode(currency_code_bytes("USD").unwrap()), "0000000000000000000000005553440000000000");
        assert_eq!(
            hex::encode(currency_code_bytes("524C555344000000000000000000000000000000").unwrap()),
            "524c555344000000000000000000000000000000"
        );
        assert_eq!(hex::encode(currency_code_bytes("RLUSD").unwrap()), "524c555344000000000000000000000000000000");
    }

    #[test]
    fn test_issued_amount_precision_ignores_trailing_zeros() {
        assert_eq!(issued_amount_precision(&BigDecimal::from_str("1000").unwrap()), 1);
    }

    #[test]
    fn test_issued_amount_value_encoding() {
        let XrpAmount::Issued(amount) = XrpAmount::issued("10", "USD", "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap() else {
            panic!("expected issued amount");
        };
        assert_eq!(amount.encoded_value().unwrap(), 0xd4c38d7ea4c68000);

        let XrpAmount::Issued(amount) = XrpAmount::issued("29.3e-1", "USD", "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap() else {
            panic!("expected issued amount");
        };
        assert_eq!(amount.encoded_value().unwrap(), 0xd48a68d1c9312000);
    }
}
