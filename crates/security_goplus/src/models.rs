use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T> {
    pub code: i32,
    pub message: String,
    pub result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAddress {
    pub cybercrime: String,
    pub money_laundering: String,
    pub financial_crime: String,
    pub blacklist_doubt: String,
    pub stealing_attack: String,
}

impl SecurityAddress {
    pub fn is_malicious(&self) -> bool {
        self.cybercrime == "1" || self.money_laundering == "1" || self.financial_crime == "1" || self.blacklist_doubt == "1" || self.stealing_attack == "1"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityToken {
    #[serde(default)]
    pub is_honeypot: Option<String>,
    #[serde(default)]
    pub fake_token: Option<String>,
    #[serde(default)]
    pub is_airdrop_scam: Option<String>,
    #[serde(default)]
    pub cannot_buy: Option<String>,
    #[serde(default)]
    pub cannot_sell_all: Option<String>,
    #[serde(default)]
    pub is_blacklisted: Option<String>,
}

impl SecurityToken {
    pub fn is_malicious(&self) -> bool {
        self.is_honeypot.as_deref() == Some("1")
            || self.fake_token.as_deref() == Some("1")
            || self.is_airdrop_scam.as_deref() == Some("1")
            || self.cannot_buy.as_deref() == Some("1")
            || self.cannot_sell_all.as_deref() == Some("1")
            || self.is_blacklisted.as_deref() == Some("1")
    }
}
