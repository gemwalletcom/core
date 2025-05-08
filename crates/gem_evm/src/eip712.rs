use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str_or_int;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EIP712Domain {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub version: String,
    #[serde(rename = "chainId")]
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub chain_id: u64,
    #[serde(rename = "verifyingContract")]
    pub verifying_contract: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EIP712Type {
    pub name: String,
    pub r#type: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EIP712Field {
    pub name: String,
    pub value: EIP712TypedValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EIP712TypedValue {
    Address { value: String },
    Uint256 { value: String }, // Represent all uint<N> as string for simplicity
    String { value: String },
    Bool { value: bool },
    Bytes { value: Vec<u8> }, // Represent all bytes<N> and dynamic bytes as Vec<u8>
    Struct { fields: Vec<EIP712Field> },
    Array { items: Vec<EIP712TypedValue> },
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
