use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_serializers::deserialize_u64_from_str_or_int;
use signer::hash_eip712;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EIP712Domain {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub version: Option<String>,
    #[serde(rename = "chainId")]
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub chain_id: u64,
    #[serde(rename = "verifyingContract")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub verifying_contract: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub salts: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EIP712Type {
    pub name: String,
    pub r#type: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EIP712Field {
    pub name: String,
    pub value: EIP712TypedValue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EIP712TypedValue {
    Address { value: String },
    Uint256 { value: String },
    String { value: String },
    Bool { value: bool },
    Bytes { value: Vec<u8> },
    Struct { fields: Vec<EIP712Field> },
    Array { items: Vec<EIP712TypedValue> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EIP712Message {
    pub domain: EIP712Domain,
    pub primary_type: String,
    pub message: Vec<EIP712Field>,
}

pub fn eip712_domain_types() -> Vec<EIP712Type> {
    vec![
        EIP712Type {
            name: "name".into(),
            r#type: "string".into(),
        },
        EIP712Type {
            name: "version".into(),
            r#type: "string".into(),
        },
        EIP712Type {
            name: "chainId".into(),
            r#type: "uint256".into(),
        },
        EIP712Type {
            name: "verifyingContract".into(),
            r#type: "address".into(),
        },
    ]
}

pub fn eip712_hash_message(value: Value) -> Result<Vec<u8>, String> {
    let json = serde_json::to_string(&value).map_err(|e| format!("Invalid EIP712 JSON: serialize error: {e}"))?;
    hash_eip712(&json).map(|digest| digest.to_vec()).map_err(|e| e.to_string())
}

pub fn parse_eip712_json(value: &Value) -> Result<EIP712Message, String> {
    let domain_value = value.get("domain").ok_or_else(|| "Invalid EIP712 JSON: missing domain".to_string())?;
    let domain: EIP712Domain = serde_json::from_value(domain_value.clone()).map_err(|e| format!("Invalid EIP712 JSON: domain parse error: {e}"))?;

    let types_value = value
        .get("types")
        .and_then(Value::as_object)
        .ok_or_else(|| "Invalid EIP712 JSON: missing or invalid types".to_string())?;
    let all_types: HashMap<String, Vec<EIP712Type>> = types_value
        .iter()
        .map(|(k, v)| {
            serde_json::from_value(v.clone())
                .map(|fields| (k.clone(), fields))
                .map_err(|e| format!("Invalid EIP712 JSON: types field '{k}' parse error: {e}"))
        })
        .collect::<Result<_, _>>()?;

    let primary_type_name = value
        .get("primaryType")
        .and_then(Value::as_str)
        .ok_or_else(|| "Invalid EIP712 JSON: missing or invalid primaryType".to_string())?;

    let message_json_value = value.get("message").ok_or_else(|| "Invalid EIP712 JSON: missing message".to_string())?;

    let message_typed_value = parse_value(primary_type_name, message_json_value, &all_types)?;

    let message_fields = match message_typed_value {
        EIP712TypedValue::Struct { fields } => fields,
        _ => return Err(format!("Primary type '{primary_type_name}' did not resolve to a Struct")),
    };

    Ok(EIP712Message {
        domain,
        primary_type: primary_type_name.to_string(),
        message: message_fields,
    })
}

pub fn parse_value(type_name: &str, json_value: &Value, all_types: &HashMap<String, Vec<EIP712Type>>) -> Result<EIP712TypedValue, String> {
    // 1. Handle Arrays
    if let Some(base_type) = type_name.strip_suffix("[]") {
        let items_json = json_value
            .as_array()
            .ok_or_else(|| format!("Expected array for type '{type_name}', got: {json_value:?}"))?;
        let mut items = Vec::with_capacity(items_json.len());
        for item_json in items_json {
            items.push(parse_value(base_type, item_json, all_types)?);
        }
        Ok(EIP712TypedValue::Array { items })
    } else {
        // 2. Handle Non-Array Types
        match type_name {
            "address" => {
                let s = json_value.as_str().ok_or_else(|| format!("Expected string for address, got: {json_value:?}"))?;
                Ok(EIP712TypedValue::Address { value: s.to_string() })
            }
            "string" => {
                let s = json_value
                    .as_str()
                    .ok_or_else(|| format!("Expected string for string type, got: {json_value:?}"))?;
                Ok(EIP712TypedValue::String { value: s.to_string() })
            }
            "bool" => {
                let b = json_value
                    .as_bool()
                    .ok_or_else(|| format!("Expected boolean for bool type, got: {json_value:?}"))?;
                Ok(EIP712TypedValue::Bool { value: b })
            }
            "bytes" => {
                // Dynamic bytes
                let s = json_value
                    .as_str()
                    .ok_or_else(|| format!("Expected hex string for bytes type, got: {json_value:?}"))?;
                let bytes_vec = hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(|e| format!("Invalid hex string for bytes type: {s}, error: {e}"))?;
                Ok(EIP712TypedValue::Bytes { value: bytes_vec })
            }
            // Wildcard for uint<N>, bytes<N>, and structs
            other_type_name => {
                if other_type_name.starts_with("uint") {
                    let value_str = match json_value {
                        Value::Number(n) => n.to_string(),
                        Value::String(s) => s.clone(),
                        _ => return Err(format!("Expected number or string for uint type '{other_type_name}', got: {json_value:?}")),
                    };
                    Ok(EIP712TypedValue::Uint256 { value: value_str })
                } else if other_type_name.starts_with("bytes") {
                    // Fixed-size bytes<N>
                    let s = json_value
                        .as_str()
                        .ok_or_else(|| format!("Expected hex string for bytes type '{other_type_name}', got: {json_value:?}"))?;
                    let bytes_vec = hex::decode(s.strip_prefix("0x").unwrap_or(s))
                        .map_err(|e| format!("Invalid hex string for bytes type '{other_type_name}': {s}, error: {e}"))?;
                    Ok(EIP712TypedValue::Bytes { value: bytes_vec })
                } else {
                    // Assume it's a struct type defined in 'all_types'
                    let defined_fields = all_types
                        .get(other_type_name)
                        .ok_or_else(|| format!("Unknown or unsupported type '{other_type_name}'"))?;

                    let message_obj = json_value
                        .as_object()
                        .ok_or_else(|| format!("Expected object for struct type '{other_type_name}', got: {json_value:?}"))?;

                    let mut struct_fields = Vec::with_capacity(defined_fields.len());
                    for field_def in defined_fields {
                        let field_json_value = message_obj
                            .get(&field_def.name)
                            .ok_or_else(|| format!("Missing field '{}' for struct type '{}'", field_def.name, other_type_name))?;

                        // Recursive call for the struct field's type
                        let field_typed_value = parse_value(&field_def.r#type, field_json_value, all_types)?;

                        struct_fields.push(EIP712Field {
                            name: field_def.name.clone(),
                            value: field_typed_value,
                        });
                    }
                    Ok(EIP712TypedValue::Struct { fields: struct_fields })
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
        let json_str = include_str!("../testdata/uniswap_permit2.json");
        let value = serde_json::from_str(json_str).unwrap();
        let result = parse_eip712_json(&value);

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let message = result.unwrap();

        assert!(message.domain.chain_id == 1);
        assert_eq!(message.message.len(), 3);

        match &message.message[0].value {
            EIP712TypedValue::Struct { fields } => {
                assert_eq!(fields.len(), 4); // token, amount, expiration, nonce

                // 1.1 token (address)
                assert_eq!(fields[0].name, "token");
                match &fields[0].value {
                    EIP712TypedValue::Address { value } => assert_eq!(value, "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
                    _ => panic!("Incorrect type for details.token"),
                }
                // 1.2 amount (uint160 - parsed as Uint256 for now)
                // We parse uint160 as Uint256 { value: String } because the JSON value is a string.
                assert_eq!(fields[1].name, "amount");
                match &fields[1].value {
                    EIP712TypedValue::Uint256 { value } => assert_eq!(value, "1461501637330902918203684832716283019655932542975"),
                    _ => panic!("Incorrect type for details.amount"),
                }
                // 1.3 expiration (uint48 - parsed as Uint256 for now)
                assert_eq!(fields[2].name, "expiration");
                match &fields[2].value {
                    EIP712TypedValue::Uint256 { value } => assert_eq!(value, "1732780554"),
                    _ => panic!("Incorrect type for details.expiration"),
                }
                // 1.4 nonce (uint48 - parsed as Uint256 for now)
                assert_eq!(fields[3].name, "nonce");
                match &fields[3].value {
                    EIP712TypedValue::Uint256 { value } => assert_eq!(value, "0"),
                    _ => panic!("Incorrect type for details.nonce"),
                }
            }
            _ => panic!("Incorrect type for details field"),
        }

        assert_eq!(message.message[1].name, "spender");
        match &message.message[1].value {
            EIP712TypedValue::Address { value } => {
                assert_eq!(value.to_lowercase(), "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad");
            }
            _ => panic!("Expected spender field to be an Address"),
        }

        assert_eq!(message.message[2].name, "sigDeadline");
        match &message.message[2].value {
            EIP712TypedValue::Uint256 { value } => {
                assert_eq!(value, "1730190354");
            }
            _ => panic!("Expected sigDeadline field to be a Uint256"),
        }
    }

    #[test]
    fn test_1inch_permit_json_parsing() {
        let json_str = include_str!("../testdata/1inch_permit.json");
        let value = serde_json::from_str(json_str).unwrap();
        let result = parse_eip712_json(&value);

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let message = result.unwrap();

        assert!(message.domain.chain_id == 1);
        assert!(message.message.len() == 5);
    }
}
