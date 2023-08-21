use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum BlockchainType {
    Ethereum,
    Bitcoin,
    Binance,
    Solana,
    Cosmos,
    Ton,
    Tron,
    Aptos,
    Sui,
}