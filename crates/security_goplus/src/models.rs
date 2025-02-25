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
