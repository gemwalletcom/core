use gem_evm::{EIP712Domain, EIP712TypedValue, eip712::parse_eip712_json};
use primitives::hex;

type GemEIP712MessageDomain = EIP712Domain;

#[uniffi::remote(Record)]
pub struct GemEIP712MessageDomain {
    pub name: String,
    pub version: Option<String>,
    pub chain_id: u64,
    pub verifying_contract: Option<String>,
    pub salts: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, uniffi::Record)]
pub struct GemEIP712Message {
    pub domain: GemEIP712MessageDomain,
    pub message: Vec<GemEIP712Section>,
}

#[derive(Debug, PartialEq, uniffi::Record)]
pub struct GemEIP712Section {
    pub name: String,
    pub values: Vec<GemEIP712Value>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemEIP712ValueType {
    Text,
    Address,
    Timestamp,
}

#[derive(Debug, PartialEq, uniffi::Record)]
pub struct GemEIP712Value {
    pub name: String,
    pub value: String,
    pub value_type: GemEIP712ValueType,
}

const TIMESTAMP_FIELDS: &[&str] = &["deadline", "sigdeadline", "expiration", "validto", "validuntil", "expiry", "timestamp"];

impl GemEIP712Message {
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let value = serde_json::from_str(json_str).map_err(|error| error.to_string())?;
        let message = parse_eip712_json(&value)?;

        let mut section = GemEIP712Section {
            name: message.primary_type,
            values: vec![],
        };
        for field in &message.message {
            flatten_field(&field.name, &field.value, &mut section.values);
        }

        Ok(Self {
            domain: message.domain,
            message: vec![section],
        })
    }
}

fn flatten_field(name: &str, value: &EIP712TypedValue, out: &mut Vec<GemEIP712Value>) {
    match value {
        EIP712TypedValue::Address { value } => {
            out.push(GemEIP712Value {
                name: name.to_string(),
                value: value.clone(),
                value_type: GemEIP712ValueType::Address,
            });
        }
        EIP712TypedValue::Uint256 { value } | EIP712TypedValue::Int256 { value } | EIP712TypedValue::String { value } => {
            let value_type = if is_timestamp_field(name) {
                GemEIP712ValueType::Timestamp
            } else {
                GemEIP712ValueType::Text
            };
            out.push(GemEIP712Value {
                name: name.to_string(),
                value: value.clone(),
                value_type,
            });
        }
        EIP712TypedValue::Bool { value } => {
            out.push(GemEIP712Value {
                name: name.to_string(),
                value: value.to_string(),
                value_type: GemEIP712ValueType::Text,
            });
        }
        EIP712TypedValue::Bytes { value } => {
            out.push(GemEIP712Value {
                name: name.to_string(),
                value: hex::encode_with_0x(value),
                value_type: GemEIP712ValueType::Text,
            });
        }
        EIP712TypedValue::Struct { fields } => {
            for field in fields {
                let field_name = format!("{name}.{}", field.name);
                flatten_field(&field_name, &field.value, out);
            }
        }
        EIP712TypedValue::Array { items } => {
            let use_index = items.len() > 1;
            for (i, item) in items.iter().enumerate() {
                let index = i + 1;
                match item {
                    EIP712TypedValue::Struct { fields } => {
                        for field in fields {
                            let field_name = if use_index {
                                format!("{name}[{index}].{}", field.name)
                            } else {
                                format!("{name}.{}", field.name)
                            };
                            flatten_field(&field_name, &field.value, out);
                        }
                    }
                    _ => {
                        let item_name = if use_index { format!("{name}[{index}]") } else { name.to_string() };
                        flatten_field(&item_name, item, out);
                    }
                }
            }
        }
    }
}

fn is_timestamp_field(name: &str) -> bool {
    let base = name.rsplit('.').next().unwrap_or(name);
    let base = base.split('[').next().unwrap_or(base);
    let lower = base.to_lowercase();
    TIMESTAMP_FIELDS.iter().any(|&f| lower == f)
}

#[cfg(test)]
mod tests {
    use super::GemEIP712Message;

    #[test]
    fn from_json_returns_error_for_malformed_json() {
        assert!(GemEIP712Message::from_json("{").is_err());
    }
}
