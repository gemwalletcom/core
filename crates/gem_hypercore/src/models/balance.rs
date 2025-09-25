use gem_evm::ethereum_address_checksum;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub coin: String,
    pub token: u32,
    pub total: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balances {
    pub balances: Vec<Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tokens {
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub name: String,
    pub wei_decimals: i32,
    pub index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakeBalance {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub delegated: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub undelegated: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub total_pending_withdrawal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegationBalance {
    pub validator: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub amount: f64,
    pub locked_until_timestamp: u64,
}

impl DelegationBalance {
    pub fn validator_address(&self) -> String {
        ethereum_address_checksum(&self.validator).unwrap_or(self.validator.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Validator {
    pub validator: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub commission: f64,
    pub is_active: bool,
    pub stats: Vec<(String, ValidatorStats)>,
}

impl Validator {
    pub fn validator_address(&self) -> String {
        ethereum_address_checksum(&self.validator).unwrap_or(self.validator.clone())
    }
}

impl Validator {
    pub fn max_apr(validators: Vec<Validator>) -> f64 {
        validators
            .into_iter()
            .filter(|x| x.is_active)
            .map(|x| x.stats.into_iter().map(|(_, stat)| stat.predicted_apr).fold(0.0, f64::max))
            .fold(0.0, f64::max)
            * 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorStats {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub predicted_apr: f64,
}
