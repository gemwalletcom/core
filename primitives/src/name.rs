use typeshare::typeshare;
use crate::chain::Chain;
use serde::Serialize;

#[typeshare(swift="Codable")]
#[allow(dead_code)]
struct NameRecord {
    name: String,
    chain: Chain,
    address: String,
    provider: NameProvider,
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