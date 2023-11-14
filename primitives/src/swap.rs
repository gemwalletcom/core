use typeshare::typeshare;
use serde::{Serialize, Deserialize};

use crate::{AssetId, ChainType};

#[typeshare(swift = "Equatable, Codable")]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SwapMode {
    ExactIn,
    ExactOut, 
}

impl Default for SwapMode {
    fn default() -> Self { SwapMode::ExactIn }
}

#[derive(rocket::form::FromForm)]
#[derive(Debug, Serialize, Deserialize)]
#[typeshare()]
#[serde(rename_all = "camelCase")]
pub struct SwapQuoteRequest {
    #[field(name = "fromAsset")]
    pub from_asset: String,
    #[field(name = "toAsset")]
    pub to_asset: String,
    #[field(name = "walletAddress")]
    pub wallet_address: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapQuoteProtocolRequest {
    pub from_asset: AssetId,
    pub to_asset: AssetId,
    pub wallet_address: String,
    pub amount: String,
    pub mode: SwapMode,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift="Codable")]
pub struct SwapQuoteResult {
    pub quote: SwapQuote,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift="Codable")]
pub struct SwapQuote {
    pub chain_type: ChainType,
    pub data: SwapQuoteEthereumData,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift="Codable")]
pub struct SwapQuoteEthereumData {
    pub to: String,
    pub value: String,
    pub data: String,
}