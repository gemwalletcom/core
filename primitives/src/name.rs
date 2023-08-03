use typeshare::typeshare;
use crate::chain::Chain;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[typeshare(swift="Codable")]
#[allow(dead_code)]
pub struct NameRecord {
    pub name: String,
    pub chain: Chain,
    pub address: String,
    pub provider: NameProvider,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NameProvider {
    Ud,
    Ens,
}

impl NameProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ud => "ud",
            Self::Ens => "ens",
        }
    }
}