use crate::model::FiatProviderAsset;
use crate::model::{FiatClient, FiatMapping};
use async_trait::async_trait;
use primitives::Chain;
use primitives::{
    fiat_provider::FiatProviderName, fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest,
};
use reqwest::{self, Client};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::Deserialize;
use url::Url;

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
        let assets = self
            .get_assets(request.clone().fiat_currency, request.clone().ip_address)
            .await?
            .assets;

        let crypto_asset_symbol = format!(
            "{}_{}",
            request_map.symbol,
            request_map.network.unwrap_or_default()
        );

        if !assets
            .iter()
            .any(|x| x.crypto_asset_symbol() == crypto_asset_symbol)
        {
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

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_assets("USD".to_string(), "127.0.0.0".to_string())
            .await?
            .assets
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }
}

impl RampClient {
    pub fn new(client: Client, api_key: String) -> RampClient {
        RampClient { client, api_key }
    }

    async fn get_assets(
        &self,
        currency: String,
        ip_address: String,
    ) -> Result<QuoteAssets, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/host-api/v3/assets?currencyCode={}&userIp={}&withDisabled=false&withHidden=false",
            RAMP_API_BASE_URL, currency, ip_address
        );
        let assets = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<QuoteAssets>()
            .await?;
        Ok(assets)
    }

    pub fn map_asset(asset: QuoteAsset) -> Option<FiatProviderAsset> {
        let chain = Self::map_asset_chain(asset.chain.clone())?;
        let token_id = asset.address.clone();
        Some(FiatProviderAsset {
            chain,
            token_id,
            symbol: asset.symbol,
            network: Some(asset.chain),
        })
    }

    pub fn map_asset_chain(chain: String) -> Option<Chain> {
        match chain.as_str() {
            "ETH" => Some(Chain::Ethereum),
            "SOLANA" => Some(Chain::Solana),
            "OPTIMISM" => Some(Chain::Optimism),
            "MATIC" => Some(Chain::Polygon),
            "XRP" => Some(Chain::Xrp),
            "TRON" => Some(Chain::Tron),
            "ARBITRUM" => Some(Chain::Arbitrum),
            "BASE" => Some(Chain::Base),
            "LTC" => Some(Chain::Litecoin),
            "AVAX" => Some(Chain::AvalancheC),
            "BSC" => Some(Chain::SmartChain),
            "COSMOS" => Some(Chain::Cosmos),
            "BTC" => Some(Chain::Bitcoin),
            "DOGE" => Some(Chain::Doge),
            _ => None,
        }
    }

    async fn get_client_quote(
        &self,
        request: QuoteRequest,
    ) -> Result<Quote, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/host-api/v3/onramp/quote/all?hostApiKey={}",
            RAMP_API_BASE_URL, self.api_key
        );
        let quote = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<Quote>()
            .await?;
        Ok(quote)
    }

    fn get_fiat_quote(&self, request: FiatBuyRequest, quote: Quote) -> FiatQuote {
        let mut crypto_amount =
            Decimal::from_str(quote.clone().card_payment.crypto_amount.as_str()).unwrap();
        crypto_amount
            .set_scale(quote.asset.decimals)
            .unwrap_or_default();

        FiatQuote {
            provider: self.name().as_fiat_provider(),
            fiat_amount: request.clone().fiat_amount,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount: crypto_amount.to_f64().unwrap_or_default(),
            redirect_url: self.redirect_url(request.clone(), quote.clone()),
        }
    }

    pub fn redirect_url(&self, request: FiatBuyRequest, quote: Quote) -> String {
        let mut components = Url::parse(RAMP_REDIRECT_URL).unwrap();
        components
            .query_pairs_mut()
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
    pub symbol: String,
    pub chain: String,
    pub decimals: u32,
    pub address: Option<String>,
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
