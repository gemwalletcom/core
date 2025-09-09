use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub alpha2: String,
    pub is_allowed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPayIpAddress {
    pub alpha2: String,
    pub state: String,
    pub is_buy_allowed: bool,
    pub is_sell_allowed: bool,
    pub is_allowed: bool,
}
