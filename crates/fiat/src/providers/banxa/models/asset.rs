use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Coin {
    pub id: String,
    pub blockchain: String,
    pub network: String,
}

#[derive(Debug, Deserialize)]
pub struct Coins {
    pub coins: Vec<Asset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub id: String,
    pub blockchains: Vec<Blockchain>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Blockchain {
    pub id: String,
    pub address: Option<String>,
    pub unsupported_countries: UnsupportedCountries,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum UnsupportedCountries {
    Map(HashMap<String, Vec<String>>),
    Empty(Vec<()>),
}

impl UnsupportedCountries {
    pub fn list_map(self) -> HashMap<String, Vec<String>> {
        match self {
            UnsupportedCountries::Map(map) => map,
            UnsupportedCountries::Empty(_) => HashMap::new(),
        }
    }
}