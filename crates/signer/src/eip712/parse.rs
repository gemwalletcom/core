use crate::SignerError;
use alloy_primitives::{I256, U256};
use serde_json::Value;
use std::str::FromStr;

pub const MAX_WORD_BYTES: usize = 32;
pub const ADDR_LENGTH: usize = 20;

pub fn parse_array_type(type_name: &str) -> Option<(String, Option<usize>)> {
    if !type_name.ends_with(']') {
        return None;
    }

    let start = type_name.rfind('[')?;
    let length_str = &type_name[start + 1..type_name.len() - 1];
    let element_type = type_name[..start].to_string();

    let length = if length_str.is_empty() { None } else { Some(length_str.parse().ok()?) };

    Some((element_type, length))
}

pub fn base_type_name(type_name: &str) -> &str {
    type_name.split('[').next().unwrap_or(type_name)
}

pub fn parse_numeric_bits(type_name: &str, prefix: &str) -> Result<usize, SignerError> {
    let bits_part = &type_name[prefix.len()..];
    if bits_part.is_empty() {
        return Ok(256);
    }

    let bits = bits_part
        .parse::<usize>()
        .map_err(|_| SignerError::invalid_input(format!("Invalid bit size for type '{type_name}'")))?;
    if bits == 0 || bits > MAX_WORD_BYTES * 8 || bits % 8 != 0 {
        return SignerError::invalid_input_err(format!("Unsupported bit size for type '{type_name}'"));
    }
    Ok(bits)
}

pub fn parse_fixed_bytes_size(type_name: &str) -> Result<usize, SignerError> {
    let size_part = &type_name["bytes".len()..];
    if size_part.is_empty() {
        return SignerError::invalid_input_err(format!("Invalid fixed bytes type '{type_name}'"));
    }

    let size = size_part
        .parse::<usize>()
        .map_err(|_| SignerError::invalid_input(format!("Invalid length for {type_name}")))?;
    if size == 0 || size > MAX_WORD_BYTES {
        return SignerError::invalid_input_err(format!("Unsupported length for {type_name}"));
    }
    Ok(size)
}

pub fn parse_uint_value(value: Option<&Value>) -> Result<U256, SignerError> {
    match value {
        Some(Value::String(s)) => U256::from_str(s).map_err(SignerError::from),
        Some(Value::Number(num)) => {
            if let Some(u) = num.as_u64() {
                return Ok(U256::from(u));
            }

            if let Some(i) = num.as_i64() {
                if i >= 0 {
                    return Ok(U256::from(i as u64));
                }
                return SignerError::invalid_input_err("Negative numeric value provided for unsigned integer");
            }

            Err(SignerError::invalid_input("Unsupported numeric value for unsigned integer"))
        }
        Some(Value::Null) | None => Ok(U256::ZERO),
        Some(other) => Err(SignerError::invalid_input(format!("Expected integer value, got {}", other))),
    }
}

pub fn parse_int_value(value: Option<&Value>) -> Result<I256, SignerError> {
    match value {
        Some(Value::String(s)) => I256::from_str(s).map_err(SignerError::from),
        Some(Value::Number(num)) => {
            if let Some(i) = num.as_i64() {
                return I256::try_from(i as i128).map_err(SignerError::from);
            }

            if let Some(u) = num.as_u64() {
                return Ok(I256::from_raw(U256::from(u)));
            }

            Err(SignerError::invalid_input("Unsupported numeric value for signed integer"))
        }
        Some(Value::Null) | None => Ok(I256::ZERO),
        Some(other) => Err(SignerError::invalid_input(format!("Expected integer value, got {}", other))),
    }
}

pub fn left_pad(bytes: &[u8]) -> [u8; MAX_WORD_BYTES] {
    let mut out = [0u8; MAX_WORD_BYTES];
    let len = bytes.len().min(MAX_WORD_BYTES);
    out[MAX_WORD_BYTES - len..].copy_from_slice(&bytes[bytes.len() - len..]);
    out
}

pub fn right_pad(bytes: &[u8]) -> [u8; MAX_WORD_BYTES] {
    let mut out = [0u8; MAX_WORD_BYTES];
    let len = bytes.len().min(MAX_WORD_BYTES);
    out[..len].copy_from_slice(&bytes[..len]);
    out
}

pub fn adjust_signed_value(number: I256, bits: usize) -> Result<U256, SignerError> {
    if bits == 0 || bits > MAX_WORD_BYTES * 8 {
        return SignerError::invalid_input_err(format!("Unsupported bit size {bits} for signed integer"));
    }

    if number.bits() > bits as u32 {
        return SignerError::invalid_input_err(format!("Value out of range for signed integer with {bits} bits"));
    }

    if bits == MAX_WORD_BYTES * 8 {
        return Ok(number.into_raw());
    }

    if number.is_negative() {
        let abs = number.unsigned_abs();
        let modulus = U256::from(1u64) << bits;
        modulus
            .checked_sub(abs)
            .ok_or_else(|| SignerError::invalid_input("Failed to encode signed integer"))
    } else {
        Ok(number.unsigned_abs())
    }
}
