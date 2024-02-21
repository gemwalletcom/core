use crate::chain::Chain;
use serde::Serialize;
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Serialize)]
#[typeshare(swift = "Codable")]
#[allow(dead_code)]
pub struct NameRecord {
    pub name: String,
    pub chain: Chain,
    pub address: String,
    pub provider: NameProvider,
}

#[derive(Debug, Serialize, AsRefStr, EnumString)]
#[typeshare(swift = "Codable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum NameProvider {
    Ud,
    Ens,
    Sns,
    Ton,
    Tree,
    SpaceId,
    Eths,
    Did,
    Suins,
    Aptos,
    Injective,
    Icns,
    Placeholder(String),
}
