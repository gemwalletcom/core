use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Currencies {
    pub config: Config,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub crypto_currencies: Vec<Asset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub currency: String,
    pub network: String,
    pub contract: String,
}