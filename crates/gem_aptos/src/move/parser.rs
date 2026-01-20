use crate::signer::AccountAddress;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use primitives::{SignerError, decode_hex};
use serde::Deserialize;
use serde_json::Value;

use super::types::{ModuleId, StructTag, TypeTag};
use super::values::MoveValue;

const OPTION_MODULE: &str = "option";
const OPTION_STRUCT: &str = "Option";

#[derive(Deserialize)]
#[serde(untagged)]
enum NumericInput {
    Number(serde_json::Number),
    String(String),
}

impl NumericInput {
    fn as_string(&self) -> String {
        match self {
            NumericInput::Number(number) => number.to_string(),
            NumericInput::String(value) => value.clone(),
        }
    }
}

pub(crate) fn parse_function_id(function_id: &str) -> Result<(ModuleId, String), SignerError> {
    let parts: Vec<&str> = function_id.split("::").collect();
    if parts.len() != 3 {
        return Err(SignerError::InvalidInput("Invalid Aptos function id".to_string()));
    }
    let address = AccountAddress::from_hex(parts[0])?;
    let module = parts[1].to_string();
    let function = parts[2].to_string();

    Ok((ModuleId { address, name: module }, function))
}

pub(crate) fn parse_type_tag(value: &str) -> Result<TypeTag, SignerError> {
    let trimmed = value.trim();
    match trimmed {
        "bool" => Ok(TypeTag::Bool),
        "u8" => Ok(TypeTag::U8),
        "u16" => Ok(TypeTag::U16),
        "u32" => Ok(TypeTag::U32),
        "u64" => Ok(TypeTag::U64),
        "u128" => Ok(TypeTag::U128),
        "u256" => Ok(TypeTag::U256),
        "address" => Ok(TypeTag::Address),
        "signer" | "&signer" => Ok(TypeTag::Signer),
        _ if trimmed.starts_with("vector<") && trimmed.ends_with('>') => {
            let inner = &trimmed["vector<".len()..trimmed.len() - 1];
            Ok(TypeTag::Vector(Box::new(parse_type_tag(inner)?)))
        }
        _ => parse_struct_tag(trimmed).map(|tag| TypeTag::Struct(Box::new(tag))),
    }
}

pub(crate) fn infer_type_tags(arguments: &[Value]) -> Result<Vec<TypeTag>, SignerError> {
    arguments.iter().map(infer_type_tag).collect()
}

pub(crate) fn encode_argument(value: &Value, arg_type: &TypeTag) -> Result<Vec<u8>, SignerError> {
    let move_value = parse_move_value(value, arg_type)?;
    bcs::to_bytes(&move_value).map_err(|err| SignerError::InvalidInput(format!("Failed to encode Aptos argument: {err}")))
}

fn parse_struct_tag(value: &str) -> Result<StructTag, SignerError> {
    let (base, args) = if let Some(index) = find_top_level_char(value, '<') {
        if !value.ends_with('>') {
            return Err(SignerError::InvalidInput("Invalid Aptos struct tag".to_string()));
        }
        (&value[..index], Some(&value[index + 1..value.len() - 1]))
    } else {
        (value, None)
    };

    let parts: Vec<&str> = base.split("::").collect();
    if parts.len() != 3 {
        return Err(SignerError::InvalidInput("Invalid Aptos struct tag".to_string()));
    }

    let address = AccountAddress::from_hex(parts[0])?;
    let module = parts[1].to_string();
    let name = parts[2].to_string();
    let type_args = if let Some(args) = args {
        split_type_args(args)?.into_iter().map(|arg| parse_type_tag(&arg)).collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    Ok(StructTag { address, module, name, type_args })
}

fn split_type_args(input: &str) -> Result<Vec<String>, SignerError> {
    let mut args = Vec::new();
    let mut depth = 0u32;
    let mut start = 0usize;

    for (index, ch) in input.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                if depth == 0 {
                    return Err(SignerError::InvalidInput("Invalid Aptos type arguments".to_string()));
                }
                depth -= 1;
            }
            ',' if depth == 0 => {
                let arg = input[start..index].trim();
                if !arg.is_empty() {
                    args.push(arg.to_string());
                }
                start = index + 1;
            }
            _ => {}
        }
    }

    let last = input[start..].trim();
    if !last.is_empty() {
        args.push(last.to_string());
    }

    Ok(args)
}

fn find_top_level_char(input: &str, target: char) -> Option<usize> {
    let mut depth = 0u32;
    for (index, ch) in input.char_indices() {
        if depth == 0 && ch == target {
            return Some(index);
        }
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn infer_type_tag(value: &Value) -> Result<TypeTag, SignerError> {
    match value {
        Value::Bool(_) => Ok(TypeTag::Bool),
        Value::Number(_) => Ok(TypeTag::U64),
        Value::String(text) => {
            if text.trim().starts_with("0x") {
                Ok(TypeTag::Address)
            } else {
                Ok(TypeTag::U64)
            }
        }
        Value::Array(values) => infer_vector_type(values),
        Value::Null => Err(SignerError::InvalidInput("Cannot infer Aptos type from null".to_string())),
        Value::Object(_) => Err(SignerError::InvalidInput("Unsupported Aptos object argument".to_string())),
    }
}

fn infer_vector_type(values: &[Value]) -> Result<TypeTag, SignerError> {
    if values.is_empty() {
        return Ok(TypeTag::Vector(Box::new(TypeTag::U8)));
    }

    if values.iter().all(is_u8_value) {
        return Ok(TypeTag::Vector(Box::new(TypeTag::U8)));
    }

    if values.iter().all(is_address_value) {
        return Ok(TypeTag::Vector(Box::new(TypeTag::Address)));
    }

    Ok(TypeTag::Vector(Box::new(TypeTag::U64)))
}

fn is_u8_value(value: &Value) -> bool {
    match value {
        Value::Number(number) => number.as_u64().map(|num| num <= u8::MAX as u64).unwrap_or(false),
        Value::String(text) => parse_u8_from_str(text).is_ok(),
        _ => false,
    }
}

fn is_address_value(value: &Value) -> bool {
    match value {
        Value::String(text) => text.trim().starts_with("0x"),
        _ => false,
    }
}

fn parse_move_value(value: &Value, arg_type: &TypeTag) -> Result<MoveValue, SignerError> {
    match arg_type {
        TypeTag::Bool => Ok(MoveValue::Bool(parse_bool(value)?)),
        TypeTag::U8 => Ok(MoveValue::U8(parse_u8(value)?)),
        TypeTag::U16 => Ok(MoveValue::U16(parse_u16(value)?)),
        TypeTag::U32 => Ok(MoveValue::U32(parse_u32(value)?)),
        TypeTag::U64 => Ok(MoveValue::U64(parse_u64(value)?)),
        TypeTag::U128 => Ok(MoveValue::U128(parse_u128(value)?)),
        TypeTag::U256 => Ok(MoveValue::U256(parse_u256(value)?)),
        TypeTag::Address => Ok(MoveValue::Address(parse_address(value)?)),
        TypeTag::Signer => Ok(MoveValue::Signer(parse_address(value)?)),
        TypeTag::Vector(inner) => parse_vector(value, inner),
        TypeTag::Struct(tag) => parse_struct(value, tag),
    }
}

fn parse_struct(value: &Value, tag: &StructTag) -> Result<MoveValue, SignerError> {
    if is_option_struct(tag) {
        let inner = tag
            .type_args
            .first()
            .ok_or_else(|| SignerError::InvalidInput("Option type missing inner type".to_string()))?;
        if value.is_null() {
            return Ok(MoveValue::Vector(Vec::new()));
        }
        let inner_value = parse_move_value(value, inner)?;
        return Ok(MoveValue::Vector(vec![inner_value]));
    }

    Err(SignerError::InvalidInput("Unsupported Aptos struct argument".to_string()))
}

fn is_option_struct(tag: &StructTag) -> bool {
    tag.address == AccountAddress::one() && tag.module == OPTION_MODULE && tag.name == OPTION_STRUCT
}

fn parse_vector(value: &Value, inner: &TypeTag) -> Result<MoveValue, SignerError> {
    match value {
        Value::Array(values) => {
            let parsed = values.iter().map(|entry| parse_move_value(entry, inner)).collect::<Result<Vec<_>, _>>()?;
            Ok(MoveValue::Vector(parsed))
        }
        Value::String(text) if matches!(inner, TypeTag::U8) => {
            let bytes = parse_hex_bytes(text)?;
            let parsed = bytes.into_iter().map(MoveValue::U8).collect();
            Ok(MoveValue::Vector(parsed))
        }
        _ => Err(SignerError::InvalidInput("Invalid Aptos vector argument".to_string())),
    }
}

fn parse_hex_bytes(value: &str) -> Result<Vec<u8>, SignerError> {
    Ok(decode_hex(value)?)
}

fn parse_address(value: &Value) -> Result<AccountAddress, SignerError> {
    match value {
        Value::String(text) => AccountAddress::from_hex(text),
        Value::Number(number) => AccountAddress::from_hex(&number.to_string()),
        _ => Err(SignerError::InvalidInput("Invalid Aptos address argument".to_string())),
    }
}

fn parse_bool(value: &Value) -> Result<bool, SignerError> {
    match value {
        Value::Bool(value) => Ok(*value),
        Value::Number(number) => Ok(number.as_u64().unwrap_or(0) != 0),
        Value::String(text) => match text.trim().to_lowercase().as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(SignerError::InvalidInput("Invalid Aptos bool argument".to_string())),
        },
        _ => Err(SignerError::InvalidInput("Invalid Aptos bool argument".to_string())),
    }
}

fn parse_numeric_string(value: &Value, label: &str) -> Result<String, SignerError> {
    let input: NumericInput = serde_json::from_value(value.clone()).map_err(|_| SignerError::InvalidInput(format!("Invalid Aptos {label} argument")))?;
    Ok(input.as_string())
}

fn parse_u8(value: &Value) -> Result<u8, SignerError> {
    let text = parse_numeric_string(value, "u8")?;
    parse_u8_from_str(&text)
}

fn parse_u8_from_str(text: &str) -> Result<u8, SignerError> {
    if text.trim().starts_with("0x") {
        let bytes = decode_hex(text)?;
        if bytes.len() != 1 {
            return Err(SignerError::InvalidInput("Invalid Aptos u8 argument".to_string()));
        }
        Ok(bytes[0])
    } else {
        text.trim().parse::<u8>().map_err(|_| SignerError::InvalidInput("Invalid Aptos u8 argument".to_string()))
    }
}

fn parse_u16(value: &Value) -> Result<u16, SignerError> {
    let text = parse_numeric_string(value, "u16")?;
    parse_unsigned_from_str::<u16>(&text, "u16")
}

fn parse_u32(value: &Value) -> Result<u32, SignerError> {
    let text = parse_numeric_string(value, "u32")?;
    parse_unsigned_from_str::<u32>(&text, "u32")
}

fn parse_u64(value: &Value) -> Result<u64, SignerError> {
    let text = parse_numeric_string(value, "u64")?;
    parse_unsigned_from_str::<u64>(&text, "u64")
}

fn parse_u128(value: &Value) -> Result<u128, SignerError> {
    let text = parse_numeric_string(value, "u128")?;
    parse_unsigned_from_str::<u128>(&text, "u128")
}

fn parse_u256(value: &Value) -> Result<[u8; 32], SignerError> {
    let text = parse_numeric_string(value, "u256")?;
    let value = parse_big_uint_from_str(&text, "u256")?;
    let mut bytes = value.to_bytes_le();
    if bytes.len() > 32 {
        return Err(SignerError::InvalidInput("Aptos u256 argument too large".to_string()));
    }
    bytes.resize(32, 0u8);
    let mut output = [0u8; 32];
    output.copy_from_slice(&bytes);
    Ok(output)
}

fn parse_unsigned_from_str<T>(text: &str, label: &str) -> Result<T, SignerError>
where
    T: TryFrom<u128>,
{
    let value = parse_big_uint_from_str(text, label)?
        .to_u128()
        .ok_or_else(|| SignerError::InvalidInput(format!("Invalid Aptos {label} argument")))?;
    T::try_from(value).map_err(|_| SignerError::InvalidInput(format!("Invalid Aptos {label} argument")))
}

fn parse_big_uint_from_str(text: &str, label: &str) -> Result<BigUint, SignerError> {
    let trimmed = text.trim();
    if trimmed.starts_with("0x") {
        let bytes = decode_hex(trimmed)?;
        Ok(BigUint::from_bytes_be(&bytes))
    } else {
        BigUint::parse_bytes(trimmed.as_bytes(), 10).ok_or_else(|| SignerError::InvalidInput(format!("Invalid Aptos {label} argument")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vector_u8_hex() {
        let value = Value::String("0x0a0b".to_string());
        let parsed = parse_vector(&value, &TypeTag::U8).unwrap();
        match parsed {
            MoveValue::Vector(entries) => {
                assert_eq!(entries.len(), 2);
            }
            _ => panic!("Expected vector"),
        }
    }
}
