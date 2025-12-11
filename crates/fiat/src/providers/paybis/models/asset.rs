use serde::Deserialize;
use std::collections::{HashMap, HashSet};

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

#[derive(Debug, Clone, Deserialize)]
pub struct SellAssets {
    pub data: Vec<PayoutMethod>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PayoutMethod {
    pub pairs: Vec<PayoutPair>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayoutPair {
    pub from_asset_id: String,
}

impl SellAssets {
    pub fn get_crypto_codes(&self) -> HashSet<String> {
        self.data
            .iter()
            .flat_map(|method| method.pairs.iter().map(|pair| pair.from_asset_id.clone()))
            .collect()
    }
}
