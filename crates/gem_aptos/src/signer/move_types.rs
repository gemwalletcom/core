use super::address::AccountAddress;
use hex::FromHex;
use num_bigint::BigUint;
use primitives::SignerError;
use serde::{Deserialize, Serialize};
use serde::ser::{SerializeSeq, Serializer};
use serde_json::Value;

const OPTION_MODULE: &str = "option";
const OPTION_STRUCT: &str = "Option";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleId {
    pub address: AccountAddress,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructTag {
    pub address: AccountAddress,
    pub module: String,
    pub name: String,
    pub type_args: Vec<TypeTag>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeTag {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Vector(Box<TypeTag>),
    Struct(Box<StructTag>),
    U16,
    U32,
    U256,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryFunction {
    pub module: ModuleId,
    pub function: String,
    pub ty_args: Vec<TypeTag>,
    pub args: Vec<Vec<u8>>,
}

#[derive(Clone, Debug)]
enum MoveValue {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256([u8; 32]),
    Address(AccountAddress),
    Signer(AccountAddress),
    Vector(Vec<MoveValue>),
}

impl Serialize for MoveValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MoveValue::Bool(value) => serializer.serialize_bool(*value),
            MoveValue::U8(value) => serializer.serialize_u8(*value),
            MoveValue::U16(value) => serializer.serialize_u16(*value),
            MoveValue::U32(value) => serializer.serialize_u32(*value),
            MoveValue::U64(value) => serializer.serialize_u64(*value),
            MoveValue::U128(value) => serializer.serialize_u128(*value),
            MoveValue::U256(value) => value.serialize(serializer),
            MoveValue::Address(value) => value.serialize(serializer),
            MoveValue::Signer(value) => value.serialize(serializer),
            MoveValue::Vector(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
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
        split_type_args(args)?
            .into_iter()
            .map(|arg| parse_type_tag(&arg))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    Ok(StructTag {
        address,
        module,
        name,
        type_args,
    })
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
            .type_args.first()
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
            let parsed = values
                .iter()
                .map(|entry| parse_move_value(entry, inner))
                .collect::<Result<Vec<_>, _>>()?;
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
    let trimmed = value.trim();
    let hex_str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if hex_str.is_empty() {
        return Ok(Vec::new());
    }
    let padded = if hex_str.len() % 2 == 1 {
        format!("0{}", hex_str)
    } else {
        hex_str.to_string()
    };
    Vec::from_hex(padded.as_bytes()).map_err(|_| SignerError::InvalidInput("Invalid hex bytes".to_string()))
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

fn parse_u8(value: &Value) -> Result<u8, SignerError> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .and_then(|num| u8::try_from(num).ok())
            .ok_or_else(|| SignerError::InvalidInput("Invalid Aptos u8 argument".to_string())),
        Value::String(text) => parse_u8_from_str(text),
        _ => Err(SignerError::InvalidInput("Invalid Aptos u8 argument".to_string())),
    }
}

fn parse_u8_from_str(text: &str) -> Result<u8, SignerError> {
    let trimmed = text.trim();
    if trimmed.starts_with("0x") {
        u8::from_str_radix(trimmed.trim_start_matches("0x"), 16)
            .map_err(|_| SignerError::InvalidInput("Invalid Aptos u8 argument".to_string()))
    } else {
        trimmed
            .parse::<u8>()
            .map_err(|_| SignerError::InvalidInput("Invalid Aptos u8 argument".to_string()))
    }
}

fn parse_u16(value: &Value) -> Result<u16, SignerError> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .and_then(|num| u16::try_from(num).ok())
            .ok_or_else(|| SignerError::InvalidInput("Invalid Aptos u16 argument".to_string())),
        Value::String(text) => parse_unsigned_from_str::<u16>(text, "u16"),
        _ => Err(SignerError::InvalidInput("Invalid Aptos u16 argument".to_string())),
    }
}

fn parse_u32(value: &Value) -> Result<u32, SignerError> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .and_then(|num| u32::try_from(num).ok())
            .ok_or_else(|| SignerError::InvalidInput("Invalid Aptos u32 argument".to_string())),
        Value::String(text) => parse_unsigned_from_str::<u32>(text, "u32"),
        _ => Err(SignerError::InvalidInput("Invalid Aptos u32 argument".to_string())),
    }
}

fn parse_u64(value: &Value) -> Result<u64, SignerError> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| SignerError::InvalidInput("Invalid Aptos u64 argument".to_string())),
        Value::String(text) => parse_unsigned_from_str::<u64>(text, "u64"),
        _ => Err(SignerError::InvalidInput("Invalid Aptos u64 argument".to_string())),
    }
}

fn parse_u128(value: &Value) -> Result<u128, SignerError> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .map(u128::from)
            .ok_or_else(|| SignerError::InvalidInput("Invalid Aptos u128 argument".to_string())),
        Value::String(text) => parse_unsigned_from_str::<u128>(text, "u128"),
        _ => Err(SignerError::InvalidInput("Invalid Aptos u128 argument".to_string())),
    }
}

fn parse_u256(value: &Value) -> Result<[u8; 32], SignerError> {
    let text = match value {
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        _ => return Err(SignerError::InvalidInput("Invalid Aptos u256 argument".to_string())),
    };

    let trimmed = text.trim();
    let (radix, digits) = if trimmed.starts_with("0x") {
        (16, trimmed.trim_start_matches("0x"))
    } else {
        (10, trimmed)
    };

    let value = BigUint::parse_bytes(digits.as_bytes(), radix)
        .ok_or_else(|| SignerError::InvalidInput("Invalid Aptos u256 argument".to_string()))?;
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
    T: std::str::FromStr + num_traits::Num,
{
    let trimmed = text.trim();
    if trimmed.starts_with("0x") {
        T::from_str_radix(trimmed.trim_start_matches("0x"), 16)
            .map_err(|_| SignerError::InvalidInput(format!("Invalid Aptos {label} argument")))
    } else {
        trimmed
            .parse::<T>()
            .map_err(|_| SignerError::InvalidInput(format!("Invalid Aptos {label} argument")))
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
