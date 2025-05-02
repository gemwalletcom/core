use crate::GemstoneError;
use primitives::eip712::{EIP712Domain, EIP712Field, EIP712Type, EIP712TypedValue};
use serde_json::Value;
use std::collections::HashMap;

type GemEIP712MessageDomain = EIP712Domain;
type GemEIP712TypedValue = EIP712TypedValue;
type GemEIP712Field = EIP712Field;

#[uniffi::remote(Record)]
pub struct GemEIP712MessageDomain {
    pub name: String,
    pub version: String,
    pub chain_id: u32,
    pub verifying_contract: String,
}

#[uniffi::remote(Record)]
pub struct GemEIP712Field {
    pub name: String,
    pub value: GemEIP712TypedValue,
}

#[uniffi::remote(Enum)]
pub enum GemEIP712TypedValue {
    Address { value: String },
    Uint256 { value: String },
    String { value: String },
    Bool { value: bool },
    Bytes { value: Vec<u8> },
    Struct { fields: Vec<EIP712Field> },
    Array { items: Vec<EIP712TypedValue> },
}

#[derive(Debug, uniffi::Record)]
pub struct GemEIP712Message {
    pub domain: GemEIP712MessageDomain,
    pub message: Vec<GemEIP712Field>,
}

impl GemEIP712Message {
    pub fn from_json(json_str: &str) -> Result<Self, GemstoneError> {
        let value: Value = serde_json::from_str(json_str).map_err(|e| GemstoneError::from(format!("Invalid EIP712 JSON: {}", e)))?;

        let domain_value = value.get("domain").ok_or_else(|| GemstoneError::from("Invalid EIP712 JSON: missing domain"))?;
        let domain: GemEIP712MessageDomain =
            serde_json::from_value(domain_value.clone()).map_err(|e| GemstoneError::from(format!("Invalid EIP712 JSON: domain parse error: {}", e)))?;

        let types_value = value
            .get("types")
            .and_then(Value::as_object)
            .ok_or_else(|| GemstoneError::from("Invalid EIP712 JSON: missing or invalid types"))?;
        let all_types: HashMap<String, Vec<EIP712Type>> = types_value
            .iter()
            .map(|(k, v)| {
                serde_json::from_value(v.clone())
                    .map(|fields| (k.clone(), fields))
                    .map_err(|e| GemstoneError::from(format!("Invalid EIP712 JSON: types field '{}' parse error: {}", k, e)))
            })
            .collect::<Result<_, _>>()?;

        let primary_type_name = value
            .get("primaryType")
            .and_then(Value::as_str)
            .ok_or_else(|| GemstoneError::from("Invalid EIP712 JSON: missing or invalid primaryType"))?;

        let message_json_value = value
            .get("message")
            .ok_or_else(|| GemstoneError::from("Invalid EIP712 JSON: missing message"))?;

        let message_typed_value = parse_value(primary_type_name, message_json_value, &all_types)?;

        let message_fields = match message_typed_value {
            GemEIP712TypedValue::Struct { fields } => fields,
            _ => return Err(GemstoneError::from(format!("Primary type '{}' did not resolve to a Struct", primary_type_name))),
        };

        Ok(GemEIP712Message {
            domain,
            message: message_fields,
        })
    }
}

fn parse_value(type_name: &str, json_value: &Value, all_types: &HashMap<String, Vec<EIP712Type>>) -> Result<GemEIP712TypedValue, GemstoneError> {
    // 1. Handle Arrays
    if let Some(base_type) = type_name.strip_suffix("[]") {
        let items_json = json_value
            .as_array()
            .ok_or_else(|| GemstoneError::from(format!("Expected array for type '{}', got: {:?}", type_name, json_value)))?;
        let mut items = Vec::with_capacity(items_json.len());
        for item_json in items_json {
            items.push(parse_value(base_type, item_json, all_types)?);
        }
        Ok(GemEIP712TypedValue::Array { items })
    } else {
        // 2. Handle Non-Array Types
        match type_name {
            "address" => {
                let s = json_value
                    .as_str()
                    .ok_or_else(|| GemstoneError::from(format!("Expected string for address, got: {:?}", json_value)))?;
                Ok(GemEIP712TypedValue::Address { value: s.to_string() })
            }
            "string" => {
                let s = json_value
                    .as_str()
                    .ok_or_else(|| GemstoneError::from(format!("Expected string for string type, got: {:?}", json_value)))?;
                Ok(GemEIP712TypedValue::String { value: s.to_string() })
            }
            "bool" => {
                let b = json_value
                    .as_bool()
                    .ok_or_else(|| GemstoneError::from(format!("Expected boolean for bool type, got: {:?}", json_value)))?;
                Ok(GemEIP712TypedValue::Bool { value: b })
            }
            "bytes" => {
                // Dynamic bytes
                let s = json_value
                    .as_str()
                    .ok_or_else(|| GemstoneError::from(format!("Expected hex string for bytes type, got: {:?}", json_value)))?;
                let bytes_vec = hex::decode(s.strip_prefix("0x").unwrap_or(s))
                    .map_err(|e| GemstoneError::from(format!("Invalid hex string for bytes type: {}, error: {}", s, e)))?;
                Ok(GemEIP712TypedValue::Bytes { value: bytes_vec })
            }
            // Wildcard for uint<N>, bytes<N>, and structs
            other_type_name => {
                if other_type_name.starts_with("uint") {
                    let value_str = match json_value {
                        Value::Number(n) => n.to_string(),
                        Value::String(s) => s.clone(),
                        _ => {
                            return Err(GemstoneError::from(format!(
                                "Expected number or string for uint type '{}', got: {:?}",
                                other_type_name, json_value
                            )))
                        }
                    };
                    Ok(GemEIP712TypedValue::Uint256 { value: value_str })
                } else if other_type_name.starts_with("bytes") {
                    // Fixed-size bytes<N>
                    let s = json_value
                        .as_str()
                        .ok_or_else(|| GemstoneError::from(format!("Expected hex string for bytes type '{}', got: {:?}", other_type_name, json_value)))?;
                    let bytes_vec = hex::decode(s.strip_prefix("0x").unwrap_or(s))
                        .map_err(|e| GemstoneError::from(format!("Invalid hex string for bytes type '{}': {}, error: {}", other_type_name, s, e)))?;
                    Ok(GemEIP712TypedValue::Bytes { value: bytes_vec })
                } else {
                    // Assume it's a struct type defined in 'all_types'
                    let defined_fields = all_types
                        .get(other_type_name)
                        .ok_or_else(|| GemstoneError::from(format!("Unknown or unsupported type '{}'", other_type_name)))?;

                    let message_obj = json_value
                        .as_object()
                        .ok_or_else(|| GemstoneError::from(format!("Expected object for struct type '{}', got: {:?}", other_type_name, json_value)))?;

                    let mut struct_fields = Vec::with_capacity(defined_fields.len());
                    for field_def in defined_fields {
                        let field_json_value = message_obj
                            .get(&field_def.name)
                            .ok_or_else(|| GemstoneError::from(format!("Missing field '{}' for struct type '{}'", field_def.name, other_type_name)))?;

                        // Recursive call for the struct field's type
                        let field_typed_value = parse_value(&field_def.r#type, field_json_value, all_types)?;

                        struct_fields.push(GemEIP712Field {
                            name: field_def.name.clone(),
                            value: field_typed_value,
                        });
                    }
                    Ok(GemEIP712TypedValue::Struct { fields: struct_fields })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permit2_json_parsing() {
        let json_str = include_str!("./test/uniswap_permit2.json");

        let result = GemEIP712Message::from_json(json_str);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let parsed_message = result;

        assert!(parsed_message.is_ok());
    }
}
