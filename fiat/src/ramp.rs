use reqwest::{self, Client};
use serde::Deserialize;
use url::Url;
use primitives::{fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest, fiat_provider::FiatProviderName};
use crate::model::{FiatMapping, FiatClient};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use async_trait::async_trait;

pub struct RampClient {
    client: Client,
    api_key: String,
}

const RAMP_API_BASE_URL: &str = "https://api.ramp.network";
const RAMP_REDIRECT_URL: &str = "https://app.ramp.network";

#[async_trait]
impl FiatClient for RampClient {
    fn name(&self) -> FiatProviderName {
        FiatProviderName::Ramp
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self.get_assets(request.clone().fiat_currency, request.clone().ip_address).await?.assets;

        let crypto_asset_symbol = format!("{}_{}", request_map.symbol, request_map.network.unwrap_or_default());

        if !assets.iter().any(|x| x.crypto_asset_symbol() == crypto_asset_symbol) {
            return Err("asset not supported".into());
        }

        let payload = QuoteRequest {
            crypto_asset_symbol,
            fiat_currency: request.clone().fiat_currency,
            fiat_value: request.fiat_amount,
        };
        let quote = self.get_client_quote(payload).await?;

        Ok(self.get_fiat_quote(request.clone(), quote))
    }
}

impl RampClient {
    pub fn new(client: Client, api_key: String) -> RampClient {
        RampClient {
            client,
            api_key,
        }
    }

    async fn get_assets(&self, currency: String, ip_address: String) -> Result<QuoteAssets, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/host-api/v3/assets?currencyCode={}&userIp={}&withDisabled=false&withHidden=false",
            RAMP_API_BASE_URL, currency, ip_address
        );
        let assets = self.client
            .get(&url)
            .send()
            .await?
            .json::<QuoteAssets>()
            .await?;
        Ok(assets)
    }

    async fn get_client_quote(&self, request: QuoteRequest) -> Result<Quote, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/host-api/v3/onramp/quote/all?hostApiKey={}",
            RAMP_API_BASE_URL, self.api_key
        );
        let quote = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<Quote>()
            .await?;
        Ok(quote)
    }

    fn get_fiat_quote(&self, request: FiatBuyRequest, quote: Quote) -> FiatQuote {
        let mut crypto_amount = Decimal::from_str(quote.clone().card_payment.crypto_amount.as_str()).unwrap();
        crypto_amount.set_scale(quote.asset.decimals).unwrap_or_default();
        
        FiatQuote{
            provider: self.name().as_fiat_provider(),
            fiat_amount: request.clone().fiat_amount,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount: crypto_amount.to_f64().unwrap_or_default(),
            redirect_url: self.redirect_url(request.clone(), quote.clone()),
        }
    }

    pub fn redirect_url(&self, request: FiatBuyRequest, quote: Quote) -> String {
        let mut components = Url::parse(RAMP_REDIRECT_URL).unwrap();
        components.query_pairs_mut()
            .append_pair("hostApiKey", &self.api_key)
            .append_pair("defaultAsset", &quote.asset.crypto_asset_symbol())
            .append_pair("swapAsset", &quote.asset.crypto_asset_symbol())
            .append_pair("fiatCurrency", &request.clone().fiat_currency.to_string())
            .append_pair("fiatValue", &request.clone().fiat_amount.to_string())
            .append_pair("userAddress", request.wallet_address.as_str());
        
        components.as_str().to_string()
    }
}


#[derive(Debug, Deserialize, Clone)]
pub struct Quote {
    #[serde(rename = "CARD_PAYMENT")]
    card_payment: QuoteData,
    asset: QuoteAsset,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteData {
    //fiat_currency: String,
    crypto_amount: String,
    //fiat_value: u32,
    //base_ramp_fee: f64,
    //applied_fee: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteAsset {
    symbol: String,
    chain: String,
    decimals: u32,
    //enabled: bool,
	//hidden: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QuoteAssets {
    assets: Vec<QuoteAsset>,
}

impl QuoteAsset {
    pub fn crypto_asset_symbol(&self) -> String {
        format!("{}_{}", self.symbol, self.chain)
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct QuoteRequest {
    crypto_asset_symbol: String,
    fiat_currency: String,
    fiat_value: f64,
}