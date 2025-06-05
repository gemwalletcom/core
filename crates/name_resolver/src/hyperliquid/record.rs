use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    pub name: NameRecord,
    pub data: DataRecord,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NameRecord {
    pub owner: String,
    pub name: String,
    pub expiry: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRecord {
    pub chain_addresses: HashMap<String, String>,
}
