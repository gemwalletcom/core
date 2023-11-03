use serde::{Serialize, Deserialize};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum BitcoinChain {
    Bitcoin,
    Litecoin,
    Doge
}