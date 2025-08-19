use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinData {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinStore {
    pub coin: Coin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinInfo {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}
