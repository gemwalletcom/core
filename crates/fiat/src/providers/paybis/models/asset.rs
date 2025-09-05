use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Assets {
    pub meta: MetaData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetaData {
    pub currencies: Vec<Currency>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub code: String,
    pub blockchain_name: Option<String>,
}

impl Currency {
    pub fn is_crypto(&self) -> bool {
        self.blockchain_name.is_some()
    }

    pub fn unsupported_countries(&self) -> HashMap<String, Vec<String>> {
        HashMap::new()
    }
}
