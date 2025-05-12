use alloy_primitives::hex;
use gem_evm::{eip712::parse_eip712_json, EIP712Domain, EIP712TypedValue};

type GemEIP712MessageDomain = EIP712Domain;

#[uniffi::remote(Record)]
pub struct GemEIP712MessageDomain {
    pub name: String,
    pub version: String,
    pub chain_id: u64,
    pub verifying_contract: String,
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

#[derive(Debug, PartialEq, uniffi::Record)]
pub struct GemEIP712Value {
    pub name: String,
    pub value: String,
}

impl GemEIP712Message {
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let value = serde_json::from_str(json_str).unwrap();
        let message = parse_eip712_json(&value)?;

        // Only show primary type section for now
        let mut section = GemEIP712Section {
            name: message.primary_type,
            values: vec![],
        };
        for field in message.message {
            let value = match field.value {
                EIP712TypedValue::Address { value } => value,
                EIP712TypedValue::Uint256 { value } => value,
                EIP712TypedValue::String { value } => value,
                EIP712TypedValue::Bool { value } => value.to_string(),
                EIP712TypedValue::Bytes { value } => hex::encode_prefixed(&value),
                EIP712TypedValue::Struct { fields } => format!("{{{}}}", fields.into_iter().map(|field| field.name).collect::<Vec<_>>().join(", ")),
                EIP712TypedValue::Array { items: _ } => "[...]".to_string(),
            };
            section.values.push(GemEIP712Value { name: field.name, value });
        }

        Ok(Self {
            domain: message.domain,
            message: vec![section],
        })
    }
}
