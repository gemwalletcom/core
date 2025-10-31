use crate::SignerError;
use alloy_primitives::hex;
use gem_hash::keccak::keccak256;
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::collections::BTreeSet;

use super::data::{TypeField, TypedData};
use super::parse::{
    ADDR_LENGTH, MAX_WORD_BYTES, adjust_signed_value, base_type_name, left_pad, parse_array_type, parse_fixed_bytes_size, parse_int_value, parse_numeric_bits,
    parse_uint_value, right_pad,
};

const PREFIX_PERSONAL_MESSAGE: &[u8] = b"\x19\x01";

pub fn hash_typed_data(json: &str) -> Result<[u8; 32], SignerError> {
    let parsed = TypedData::from_json(json)?;

    if parsed.message.is_null() {
        return SignerError::invalid_input_err("Invalid EIP-712 JSON: missing message");
    }

    let domain_hash = if parsed.types.contains_key("EIP712Domain") {
        hash_struct("EIP712Domain", Some(&parsed.domain), &parsed.types)?
    } else {
        [0u8; 32]
    };

    let message_hash = hash_struct(&parsed.primary_type, Some(&parsed.message), &parsed.types)?;

    let mut preimage = Vec::with_capacity(2 + 32 + 32);
    preimage.extend_from_slice(PREFIX_PERSONAL_MESSAGE);
    preimage.extend_from_slice(&domain_hash);
    preimage.extend_from_slice(&message_hash);

    Ok(keccak256(&preimage))
}

fn hash_struct(primary_type: &str, data: Option<&Value>, types: &std::collections::HashMap<String, Vec<TypeField>>) -> Result<[u8; 32], SignerError> {
    let fields = types
        .get(primary_type)
        .ok_or_else(|| SignerError::invalid_input(format!("Unknown EIP-712 type '{primary_type}'")))?;

    let data_map: Cow<Map<String, Value>> = match data {
        Some(Value::Object(map)) => Cow::Borrowed(map),
        Some(Value::Null) | None => Cow::Owned(Map::new()),
        Some(other) => return Err(SignerError::invalid_input(format!("Expected object for type '{primary_type}', got {}", other))),
    };

    let mut encoded = Vec::with_capacity(32 * (fields.len() + 1));
    let type_hash = hash_type(primary_type, types)?;
    encoded.extend_from_slice(&type_hash);

    for field in fields {
        let encoded_value = encode_value(&field.r#type, data_map.get(&field.name), types)?;
        encoded.extend_from_slice(&encoded_value);
    }

    Ok(keccak256(&encoded))
}

fn encode_value(type_name: &str, value: Option<&Value>, types: &std::collections::HashMap<String, Vec<TypeField>>) -> Result<[u8; 32], SignerError> {
    if let Some((element_type, expected_len)) = parse_array_type(type_name) {
        let mut concatenated = Vec::new();

        match value {
            Some(Value::Array(items)) => {
                if let Some(len) = expected_len
                    && items.len() != len
                {
                    return Err(SignerError::invalid_input(format!(
                        "Expected array of length {len} for type '{ty}', got {}",
                        items.len(),
                        ty = type_name
                    )));
                }

                for item in items {
                    let element_bytes = encode_value(&element_type, Some(item), types)?;
                    concatenated.extend_from_slice(&element_bytes);
                }
            }
            Some(Value::Null) | None => {
                if let Some(len) = expected_len
                    && len != 0
                {
                    return Err(SignerError::invalid_input(format!(
                        "Expected array of length {len} for type '{ty}', but value was null",
                        ty = type_name
                    )));
                }
            }
            Some(other) => {
                return Err(SignerError::invalid_input(format!(
                    "Expected array for type '{ty}', got {}",
                    other,
                    ty = type_name
                )));
            }
        }

        return Ok(keccak256(&concatenated));
    }

    let base_type = base_type_name(type_name);
    if types.contains_key(base_type) {
        return hash_struct(base_type, value, types);
    }

    match base_type {
        "string" => encode_string(value),
        "bytes" => encode_bytes(value),
        "bool" => encode_bool(value),
        "address" => encode_address(value),
        _ => {
            if base_type.starts_with("uint") || base_type == "uint" {
                encode_uint(base_type, value)
            } else if base_type.starts_with("int") {
                encode_int(base_type, value)
            } else if base_type.starts_with("bytes") {
                encode_fixed_bytes(base_type, value)
            } else {
                SignerError::invalid_input_err(format!("Unsupported EIP-712 type '{ty}'", ty = type_name))
            }
        }
    }
}

fn encode_string(value: Option<&Value>) -> Result<[u8; 32], SignerError> {
    let string_value = match value {
        Some(Value::String(s)) => s.as_str(),
        Some(Value::Null) | None => "",
        Some(other) => return Err(SignerError::invalid_input(format!("Expected string value, got {}", other))),
    };

    Ok(keccak256(string_value.as_bytes()))
}

fn encode_bytes(value: Option<&Value>) -> Result<[u8; 32], SignerError> {
    let bytes = match value {
        Some(Value::String(s)) => hex::decode(s)?,
        Some(Value::Null) | None => Vec::new(),
        Some(other) => return Err(SignerError::invalid_input(format!("Expected hex string for bytes value, got {}", other))),
    };

    Ok(keccak256(&bytes))
}

fn encode_bool(value: Option<&Value>) -> Result<[u8; 32], SignerError> {
    let bool_value = match value {
        Some(Value::Bool(b)) => *b,
        Some(Value::Null) | None => false,
        Some(Value::Number(num)) => match (num.as_u64(), num.as_i64()) {
            (Some(v), _) => v != 0,
            (_, Some(v)) => v != 0,
            _ => return Err(SignerError::invalid_input("Invalid numeric value for bool")),
        },
        Some(other) => return Err(SignerError::invalid_input(format!("Expected boolean value, got {}", other))),
    };

    if bool_value { Ok(left_pad(&[1])) } else { Ok([0u8; 32]) }
}

fn encode_address(value: Option<&Value>) -> Result<[u8; 32], SignerError> {
    let bytes = match value {
        Some(Value::String(s)) => {
            let raw = hex::decode(s)?;
            if raw.len() != 20 {
                return Err(SignerError::invalid_input(format!("Invalid address length for '{s}'")));
            }
            raw
        }
        Some(Value::Null) | None => vec![0u8; ADDR_LENGTH],
        Some(other) => return Err(SignerError::invalid_input(format!("Expected address string, got {}", other))),
    };

    Ok(left_pad(&bytes))
}

fn encode_uint(type_name: &str, value: Option<&Value>) -> Result<[u8; 32], SignerError> {
    let bits = parse_numeric_bits(type_name, "uint")?;
    let number = parse_uint_value(value)?;

    if number.bit_len() > bits {
        return Err(SignerError::invalid_input(format!(
            "Value out of range for type '{type_name}' ({bits}-bit unsigned integer)"
        )));
    }

    Ok(number.to_be_bytes::<MAX_WORD_BYTES>())
}

fn encode_int(type_name: &str, value: Option<&Value>) -> Result<[u8; 32], SignerError> {
    let bits = parse_numeric_bits(type_name, "int")?;
    let number = parse_int_value(value)?;
    let unsigned = adjust_signed_value(number, bits)?;
    Ok(unsigned.to_be_bytes::<MAX_WORD_BYTES>())
}

fn encode_fixed_bytes(type_name: &str, value: Option<&Value>) -> Result<[u8; 32], SignerError> {
    let size = parse_fixed_bytes_size(type_name)?;
    let mut bytes = match value {
        Some(Value::String(s)) if s.is_empty() => Vec::new(),
        Some(Value::String(s)) => hex::decode(s)?,
        Some(Value::Null) | None => Vec::new(),
        Some(other) => return Err(SignerError::invalid_input(format!("Expected hex string for {type_name}, got {}", other))),
    };

    if bytes.len() > size {
        return Err(SignerError::invalid_input(format!("Value too large for type '{type_name}'")));
    }

    if bytes.len() < size {
        bytes.resize(size, 0u8);
    }

    Ok(right_pad(&bytes))
}

fn hash_type(primary_type: &str, types: &std::collections::HashMap<String, Vec<TypeField>>) -> Result<[u8; 32], SignerError> {
    let encoded = encode_type(primary_type, types)?;
    Ok(keccak256(encoded.as_bytes()))
}

fn encode_type(primary_type: &str, types: &std::collections::HashMap<String, Vec<TypeField>>) -> Result<String, SignerError> {
    if !types.contains_key(primary_type) {
        return Err(SignerError::invalid_input(format!("Unknown EIP-712 type '{primary_type}'")));
    }

    let mut deps = BTreeSet::new();
    collect_type_dependencies(primary_type, types, &mut deps);
    deps.remove(primary_type);

    let mut parts = Vec::with_capacity(deps.len() + 1);
    parts.push(primary_type.to_string());
    parts.extend(deps);

    let mut encoded = String::new();
    for type_name in parts {
        let fields = types
            .get(&type_name)
            .ok_or_else(|| SignerError::invalid_input(format!("Unknown EIP-712 type '{type_name}'")))?;

        encoded.push_str(&type_name);
        encoded.push('(');
        for (idx, field) in fields.iter().enumerate() {
            if idx > 0 {
                encoded.push(',');
            }
            encoded.push_str(&field.r#type);
            encoded.push(' ');
            encoded.push_str(&field.name);
        }
        encoded.push(')');
    }

    Ok(encoded)
}

fn collect_type_dependencies(primary_type: &str, types: &std::collections::HashMap<String, Vec<TypeField>>, results: &mut BTreeSet<String>) {
    let base = base_type_name(primary_type);
    if results.contains(base) || !types.contains_key(base) {
        return;
    }

    results.insert(base.to_string());

    if let Some(fields) = types.get(base) {
        for field in fields {
            collect_type_dependencies(&field.r#type, types, results);
        }
    }
}
