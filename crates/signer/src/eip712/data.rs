use crate::SignerError;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct TypeField {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct TypedData {
    pub types: HashMap<String, Vec<TypeField>>,
    #[serde(rename = "primaryType")]
    pub primary_type: String,
    #[serde(default)]
    pub domain: Value,
    pub message: Value,
}

impl TypedData {
    pub fn from_json(json: &str) -> Result<Self, SignerError> {
        serde_json::from_str(json).map_err(|err| SignerError::new(format!("Invalid EIP-712 JSON: {err}")))
    }
}
