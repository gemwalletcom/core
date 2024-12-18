use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChainType {
    Ethereum,
    Bitcoin,
    Solana,
    Cosmos,
    Ton,
    Tron,
    Aptos,
    Sui,
    Xrp,
    Near,
    Stellar,
}
