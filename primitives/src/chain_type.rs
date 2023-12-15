use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable, Hashable")]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    Ethereum,
    Bitcoin,
    Binance,
    Solana,
    Cosmos,
    Ton,
    Tron,
    Aptos,
    Sui,
    Xrp,
}
