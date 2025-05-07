use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
pub struct EIP712Domain {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub version: String,
    #[serde(rename = "chainId")]
    #[serde(deserialize_with = "deserialize_chain_id::deserialize")]
    pub chain_id: u32,
    #[serde(rename = "verifyingContract")]
    pub verifying_contract: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
pub struct EIP712Type {
    pub name: String,
    pub r#type: String,
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

mod deserialize_chain_id {
    use serde::{de, Deserialize, Deserializer};
    use serde_json::Value;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Number(num) => num
                .as_u64()
                .and_then(|n| n.try_into().ok())
                .ok_or_else(|| de::Error::custom(format!("Invalid number for chainId: {}", num))),
            Value::String(s) => {
                if let Some(hex_val) = s.strip_prefix("0x") {
                    u32::from_str_radix(hex_val, 16).map_err(|_| de::Error::custom(format!("Invalid hex string for chainId: {}", s)))
                } else {
                    s.parse::<u32>()
                        .map_err(|_| de::Error::custom(format!("Invalid decimal string for chainId: {}", s)))
                }
            }
            _ => Err(de::Error::custom("chainId must be a number or a string")),
        }
    }
}
