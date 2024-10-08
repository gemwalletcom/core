use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[typeshare(swift = "Equatable, CaseIterable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
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
}
