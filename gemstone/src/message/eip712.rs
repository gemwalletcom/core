use crate::GemstoneError;
use primitives::eip712::{EIP712Domain, EIP712Type};
use serde_json::Value;
use std::collections::HashMap;

type GemEIP712MessageDomain = EIP712Domain;

#[uniffi::remote(Record)]
pub struct GemEIP712MessageDomain {
    pub name: String,
    pub version: String,
    pub chain_id: u32,
    pub verifying_contract: String,
}

#[derive(Debug, uniffi::Record)]
pub struct GemEIP712Message {
    pub domain: GemEIP712MessageDomain,
    pub message: Vec<GemEIP712Field>,
}

#[derive(Debug, uniffi::Record)]
pub struct GemEIP712Field {
    pub name: String,
    pub value: GemEIP712TypedValue,
}

#[derive(Debug, uniffi::Enum)]
pub enum GemEIP712TypedValue {
    Address { value: String },
    Uint256 { value: String },
    String { value: String },
    Bool { value: bool },
    Bytes { value: Vec<u8> },
    Struct { fields: Vec<GemEIP712Field> },
    Array { items: Vec<GemEIP712TypedValue> },
}

impl GemEIP712Message {
    pub fn from_json(json: &str) -> Result<Self, GemstoneError> {
        let value: Value = serde_json::from_str(json).map_err(|e| GemstoneError::from(format!("Invalid EIP712 JSON: {}", e)))?;

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
        let json_str = r#"{"domain":{"name":"Permit2","chainId":1,"verifyingContract":"0x000000000022D473030F116dDEE9F6B43aC78BA3"},"types":{"EIP712Domain":[{"name":"name","type":"string"},{"name":"chainId","type":"uint256"},{"name":"verifyingContract","type":"address"}],"PermitSingle":[{"name":"details","type":"PermitDetails"},{"name":"spender","type":"address"},{"name":"sigDeadline","type":"uint256"}],"PermitDetails":[{"name":"token","type":"address"},{"name":"amount","type":"uint160"},{"name":"expiration","type":"uint48"},{"name":"nonce","type":"uint48"}]},"primaryType":"PermitSingle","message":{"details":{"token":"0xdAC17F958D2ee523a2206206994597C13D831ec7","amount":"1461501637330902918203684832716283019655932542975","expiration":"1732780554","nonce":"0"},"spender":"0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD","sigDeadline":"1730190354"}}"#;

        let result = GemEIP712Message::from_json(json_str);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let parsed_message = result.unwrap();

        // Verify Domain
        assert_eq!(parsed_message.domain.name, "Permit2".to_string());
        assert_eq!(parsed_message.domain.chain_id, 1);
        assert_eq!(
            parsed_message.domain.verifying_contract,
            "0x000000000022D473030F116dDEE9F6B43aC78BA3".to_string()
        );
        assert_eq!(parsed_message.domain.version, "".to_string());

        // Verify Message Structure (Primary Type: PermitSingle)
        assert_eq!(parsed_message.message.len(), 3); // details, spender, sigDeadline

        // 1. details (Struct: PermitDetails)
        let details_field = &parsed_message.message[0];
        assert_eq!(details_field.name, "details");
        match &details_field.value {
            GemEIP712TypedValue::Struct { fields } => {
                assert_eq!(fields.len(), 4); // token, amount, expiration, nonce

                // 1.1 token (address)
                assert_eq!(fields[0].name, "token");
                match &fields[0].value {
                    GemEIP712TypedValue::Address { value } => assert_eq!(value, "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
                    _ => panic!("Incorrect type for details.token"),
                }
                // 1.2 amount (uint160 - parsed as Uint256 for now as we don't have specific uint sizes yet)
                // Note: The 'types' defined uint160, uint48, but our current parser treats them based on JSON value type.
                // We parse uint160 as Uint256 { value: String } because the JSON value is a string.
                assert_eq!(fields[1].name, "amount");
                match &fields[1].value {
                    GemEIP712TypedValue::Uint256 { value } => assert_eq!(value, "1461501637330902918203684832716283019655932542975"),
                    _ => panic!("Incorrect type for details.amount"),
                }
                // 1.3 expiration (uint48 - parsed as Uint256 for now)
                assert_eq!(fields[2].name, "expiration");
                match &fields[2].value {
                    GemEIP712TypedValue::Uint256 { value } => assert_eq!(value, "1732780554"),
                    _ => panic!("Incorrect type for details.expiration"),
                }
                // 1.4 nonce (uint48 - parsed as Uint256 for now)
                assert_eq!(fields[3].name, "nonce");
                match &fields[3].value {
                    GemEIP712TypedValue::Uint256 { value } => assert_eq!(value, "0"),
                    _ => panic!("Incorrect type for details.nonce"),
                }
            }
            _ => panic!("Incorrect type for details field"),
        }

        // 2. spender (address)
        let spender_field = &parsed_message.message[1];
        assert_eq!(spender_field.name, "spender");
        match &spender_field.value {
            GemEIP712TypedValue::Address { value } => assert_eq!(value, "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD"),
            _ => panic!("Incorrect type for spender field"),
        }

        // 3. sigDeadline (uint256)
        let deadline_field = &parsed_message.message[2];
        assert_eq!(deadline_field.name, "sigDeadline");
        match &deadline_field.value {
            GemEIP712TypedValue::Uint256 { value } => assert_eq!(value, "1730190354"),
            _ => panic!("Incorrect type for sigDeadline field"),
        }
    }
}
