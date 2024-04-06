//TODO: remove later once get method is irrelevant
#![allow(clippy::blocks_in_conditions)]

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, ChainType};

#[typeshare(swift = "Equatable, Codable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SwapMode {
    #[default]
    ExactIn,
    ExactOut,
}

#[derive(rocket::form::FromForm, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Codable")]
#[serde(rename_all = "camelCase")]
pub struct SwapQuoteRequest {
    #[field(name = "fromAsset")]
    pub from_asset: String,
    #[field(name = "toAsset")]
    pub to_asset: String,
    #[field(name = "walletAddress")]
    pub wallet_address: String,
    #[field(name = "destinationAddress")]
    pub destination_address: Option<String>,
    pub amount: String,
    #[field(name = "includeData")]
    pub include_data: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapQuoteProtocolRequest {
    pub from_asset: AssetId,
    pub to_asset: AssetId,
    pub wallet_address: String,
    pub destination_address: String,
    pub amount: String,
    pub mode: SwapMode,
    pub include_data: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Equatable")]
pub struct SwapQuoteResult {
    pub quote: SwapQuote,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Equatable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct SwapQuote {
    pub chain_type: ChainType,
    pub from_amount: String,
    pub to_amount: String,
    pub fee_percent: f32,
    pub provider: SwapProvider,
    pub data: Option<SwapQuoteData>,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Equatable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct SwapProvider {
    pub name: String,
}

impl From<&'static str> for SwapProvider {
    fn from(name: &'static str) -> Self {
        SwapProvider {
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Equatable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct SwapQuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
}

impl SwapQuoteData {
    pub fn from_data(str: &str) -> Self {
        Self {
            to: String::default(),
            value: String::default(),
            data: str.to_string(),
        }
    }
}
