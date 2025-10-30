use crate::SignerError;
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, Signed, Zero};
use serde_json::Value;

pub const MAX_WORD_BYTES: usize = 32;
pub const ADDR_LENGTH: usize = 20;

pub fn parse_array_type(r#type: &str) -> Option<(String, Option<usize>)> {
    let start = r#type.find('[')?;
    let end = r#type[start..].find(']')? + start;
    let length_str = &r#type[start + 1..end];
    let remainder = &r#type[end + 1..];
    let element_type = format!("{}{}", &r#type[..start], remainder);

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
        .map_err(|_| SignerError::new(format!("Invalid bit size for type '{type_name}'")))?;
    if bits == 0 || bits > 256 || bits % 8 != 0 {
        return Err(SignerError::new(format!("Unsupported bit size for type '{type_name}'")));
    }
    Ok(bits)
}

pub fn parse_fixed_bytes_size(type_name: &str) -> Result<usize, SignerError> {
    let size_part = &type_name["bytes".len()..];
    if size_part.is_empty() {
        return Err(SignerError::new(format!("Invalid fixed bytes type '{type_name}'")));
    }

    let size = size_part
        .parse::<usize>()
        .map_err(|_| SignerError::new(format!("Invalid length for {type_name}")))?;
    if size == 0 || size > MAX_WORD_BYTES {
        return Err(SignerError::new(format!("Unsupported length for {type_name}")));
    }
    Ok(size)
}

pub fn parse_uint_value(value: Option<&Value>) -> Result<BigUint, SignerError> {
    match value {
        Some(Value::String(s)) => parse_biguint_from_string(s),
        Some(Value::Number(num)) => {
            if let Some(u) = num.as_u64() {
                Ok(BigUint::from(u))
            } else {
                Err(SignerError::new("Negative numeric value provided for unsigned integer"))
            }
        }
        Some(Value::Null) | None => Ok(BigUint::zero()),
        Some(other) => Err(SignerError::new(format!("Expected integer value, got {}", other))),
    }
}

pub fn parse_int_value(value: Option<&Value>) -> Result<BigInt, SignerError> {
    match value {
        Some(Value::String(s)) => parse_bigint_from_string(s),
        Some(Value::Number(num)) => {
            if let Some(i) = num.as_i64() {
                Ok(BigInt::from(i))
            } else if let Some(u) = num.as_u64() {
                Ok(BigInt::from(u))
            } else {
                Err(SignerError::new("Unsupported numeric value for signed integer"))
            }
        }
        Some(Value::Null) | None => Ok(BigInt::zero()),
        Some(other) => Err(SignerError::new(format!("Expected integer value, got {}", other))),
    }
}

pub fn parse_biguint_from_string(value: &str) -> Result<BigUint, SignerError> {
    let trimmed = value.trim();
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        BigUint::parse_bytes(trimmed.trim_start_matches("0x").trim_start_matches("0X").as_bytes(), 16)
            .ok_or_else(|| SignerError::new(format!("Invalid hex string '{value}'")))
    } else {
        BigUint::parse_bytes(trimmed.as_bytes(), 10).ok_or_else(|| SignerError::new(format!("Invalid decimal string '{value}'")))
    }
}

pub fn parse_bigint_from_string(value: &str) -> Result<BigInt, SignerError> {
    let trimmed = value.trim();
    if trimmed.starts_with("-0x") || trimmed.starts_with("-0X") {
        let magnitude = trimmed.trim_start_matches("-0x").trim_start_matches("-0X");
        let uint = BigUint::parse_bytes(magnitude.as_bytes(), 16).ok_or_else(|| SignerError::new(format!("Invalid hex string '{value}'")))?;
        Ok(BigInt::from_biguint(Sign::Minus, uint))
    } else if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        let uint = BigUint::parse_bytes(trimmed.trim_start_matches("0x").trim_start_matches("0X").as_bytes(), 16)
            .ok_or_else(|| SignerError::new(format!("Invalid hex string '{value}'")))?;
        Ok(BigInt::from_biguint(Sign::Plus, uint))
    } else {
        BigInt::parse_bytes(trimmed.as_bytes(), 10).ok_or_else(|| SignerError::new(format!("Invalid decimal string '{value}'")))
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

pub fn decode_hex_string(value: &str) -> Result<Vec<u8>, SignerError> {
    let stripped = value.trim_start_matches("0x").trim_start_matches("0X").replace('_', "");
    if stripped.len() % 2 != 0 {
        return Err(SignerError::new(format!("Hex string has odd length: '{value}'")));
    }
    hex::decode(&stripped).map_err(|err| SignerError::new(format!("Invalid hex string '{value}': {err}")))
}

pub fn adjust_signed_value(number: BigInt, bits: usize) -> Result<BigUint, SignerError> {
    let bound = BigInt::one() << (bits as u32 - 1);
    if number >= bound.clone() || number < -bound.clone() {
        return Err(SignerError::new(format!("Value out of range for signed integer with {bits} bits")));
    }

    let modulus = BigInt::one() << bits as u32;
    let adjusted = if number.is_negative() { modulus + number } else { number };
    adjusted.to_biguint().ok_or_else(|| SignerError::new("Failed to encode signed integer"))
}
