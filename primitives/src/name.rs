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
#[typeshare(swift="Codable")]
#[serde(rename_all = "lowercase")]
pub enum NameProvider {
    Ud,
    Ens,
    Sns,
    Ton,
    Tree,
    SpaceId,
    Eths,
}

impl NameProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ud => "ud",
            Self::Ens => "ens",
            Self::Sns => "sns",
            Self::Ton => "ton",
            Self::Tree => "tree",
            Self::SpaceId => "spaceid",
            Self::Eths => "eths",
        }
    }
}