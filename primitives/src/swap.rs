use typeshare::typeshare;
use serde::{Serialize, Deserialize};

use crate::{AssetId, ChainType};

#[derive(rocket::form::FromForm)]
#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift="Codable")]
pub struct SwapQuoteRequest {
    pub from_asset: String,
    pub to_asset: String,
    pub wallet_address: String,
    pub from_amount: String,
    pub to_amount: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapQuoteProtocolRequest {
    pub from_asset: AssetId,
    pub to_asset: AssetId,
    pub wallet_address: String,
    pub from_amount: String,
    pub to_amount: Option<String>,
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