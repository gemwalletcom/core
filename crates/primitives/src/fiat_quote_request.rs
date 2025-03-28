use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Asset, FiatQuoteType};

pub enum FiatQuoteAmount {
    Fiat(f64),
    Crypto(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteRequest {
    pub asset_id: String,
    #[serde(rename = "type")]
    pub quote_type: FiatQuoteType,
    #[typeshare(skip)]
    pub ip_address: String,
    pub fiat_currency: String,
    pub fiat_amount: Option<f64>,
    pub crypto_value: Option<String>,
    pub wallet_address: String,
    #[typeshare(skip)]
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatBuyQuote {
    pub asset: Asset,
    pub asset_id: String,
    pub ip_address: String,
    pub fiat_currency: String,
    pub fiat_amount: f64,
    pub fiat_value: String,
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatSellQuote {
    pub asset: Asset,
    pub asset_id: String,
    pub ip_address: String,
    pub fiat_currency: String,
    pub crypto_amount: f64,
    pub crypto_value: String,
    pub wallet_address: String,
}

pub enum FiatQuoteTypeResult {
    Buy(FiatBuyQuote),
    Sell(FiatSellQuote),
}

impl FiatQuoteTypeResult {
    pub fn get_wallet_address(&self) -> String {
        match self {
            FiatQuoteTypeResult::Buy(quote) => quote.wallet_address.clone(),
            FiatQuoteTypeResult::Sell(quote) => quote.wallet_address.clone(),
        }
    }
}

impl FiatQuoteRequest {
    pub fn get_buy_quote(&self, asset: Asset, fiat_value: String) -> FiatBuyQuote {
        FiatBuyQuote {
            asset,
            asset_id: self.asset_id.clone(),
            ip_address: self.ip_address.clone(),
            fiat_currency: self.fiat_currency.clone(),
            fiat_amount: self.fiat_amount.unwrap_or_default(),
            fiat_value,
            wallet_address: self.wallet_address.clone(),
        }
    }

    pub fn get_sell_quote(&self, asset: Asset, crypto_amount: f64) -> FiatSellQuote {
        FiatSellQuote {
            asset,
            asset_id: self.asset_id.clone(),
            ip_address: self.ip_address.clone(),
            fiat_currency: self.fiat_currency.clone(),
            crypto_amount,
            crypto_value: self.crypto_value.clone().unwrap_or_default(),
            wallet_address: self.wallet_address.clone(),
        }
    }
}
