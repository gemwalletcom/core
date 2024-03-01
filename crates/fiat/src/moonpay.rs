use crate::model::{FiatClient, FiatMapping, FiatProviderAsset};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use primitives::{
    fiat_provider::FiatProviderName, fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest,
    Chain,
};
use reqwest::Client;
use serde::Deserialize;
use sha2::Sha256;
use url::Url;

const MOONPAY_API_BASE_URL: &str = "https://api.moonpay.com";
const MOONPAY_REDIRECT_URL: &str = "https://buy.moonpay.com";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPayBuyQuote {
    pub quote_currency_amount: f64,
    pub quote_currency_code: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPayIpAddress {
    pub is_buy_allowed: bool,
    pub is_allowed: bool,
}

pub struct MoonPayClient {
    client: Client,
    api_key: String,
    secret_key: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub code: String,
    pub metadata: Option<CurrencyMetadata>,
    pub is_suspended: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyMetadata {
    pub contract_address: Option<String>,
    pub network_code: String,
}

#[async_trait]
impl FiatClient for MoonPayClient {
    fn name(&self) -> FiatProviderName {
        FiatProviderName::MoonPay
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let ip_address_check = self.get_ip_address(request.clone().ip_address).await?;
        if !ip_address_check.is_allowed && !ip_address_check.is_buy_allowed {
            return Err("purchase is not allowed".into());
        }

        let url = format!(
            "{}/v3/currencies/{}/buy_quote/?baseCurrencyCode={}&baseCurrencyAmount={}&areFeesIncluded={}&apiKey={}",
            MOONPAY_API_BASE_URL,
            request_map.symbol.to_lowercase(),
            request.fiat_currency.to_lowercase(),
            request.fiat_amount,
            "true",
            self.api_key,
        );

        let quote = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<MoonPayBuyQuote>()
            .await?;

        Ok(self.get_fiat_quote(request, quote))
    }

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_assets()
            .await?
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }
}

impl MoonPayClient {
    pub fn new(client: Client, api_key: String, secret_key: String) -> Self {
        Self {
            client,
            api_key,
            secret_key,
        }
    }

    pub async fn get_ip_address(
        &self,
        ip_address: String,
    ) -> Result<MoonPayIpAddress, reqwest::Error> {
        let url = format!(
            "{}/v4/ip_address/?ipAddress={}&apiKey={}",
            MOONPAY_API_BASE_URL, ip_address, self.api_key,
        );

        let response = self.client.get(&url).send().await?;
        let ip_address_result = response.json::<MoonPayIpAddress>().await?;

        Ok(ip_address_result)
    }

    pub async fn get_assets(&self) -> Result<Vec<Asset>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/v3/currencies", MOONPAY_API_BASE_URL);
        let assets = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Vec<Asset>>()
            .await?;
        Ok(assets)
    }

    pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
        // include some, bttc missing contract address
        if ["bttc"].contains(&asset.code.as_str()) {
            return None;
        }
        let chain = Self::map_asset_chain(asset.clone())?;
        let token_id = asset
            .clone()
            .metadata?
            .contract_address
            .filter(|contract_address| {
                ![
                    "0x0000000000000000000000000000000000001010",
                    "0x0000000000000000000000000000000000000000",
                ]
                .contains(&contract_address.as_str())
            });
        let enabled = !asset.is_suspended.unwrap_or(true);
        Some(FiatProviderAsset {
            chain,
            token_id,
            symbol: asset.code,
            network: None,
            enabled,
        })
    }

    pub fn map_asset_chain(asset: Asset) -> Option<Chain> {
        match asset.metadata?.network_code.as_str() {
            "ethereum" => Some(Chain::Ethereum),
            "binance_smart_chain" => Some(Chain::SmartChain),
            "solana" => Some(Chain::Solana),
            "arbitrum" => Some(Chain::Arbitrum),
            "base" => Some(Chain::Base),
            "avalanche_c_chain" => Some(Chain::AvalancheC),
            "optimism" => Some(Chain::Optimism),
            "polygon" => Some(Chain::Polygon),
            "tron" => Some(Chain::Tron),
            "aptos" => Some(Chain::Aptos),
            "bitcoin" => Some(Chain::Bitcoin),
            "dogecoin" => Some(Chain::Doge),
            "litecoin" => Some(Chain::Litecoin),
            "ripple" => Some(Chain::Xrp),
            "sui" => Some(Chain::Sui),
            "ton" => Some(Chain::Ton),
            "cosmos" => Some(Chain::Cosmos),
            _ => None,
        }
    }

    fn get_fiat_quote(&self, request: FiatBuyRequest, quote: MoonPayBuyQuote) -> FiatQuote {
        FiatQuote {
            provider: self.name().as_fiat_provider(),
            fiat_amount: request.clone().fiat_amount,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount: quote.quote_currency_amount,
            redirect_url: self.redirect_url(request.clone(), quote),
        }
    }

    pub fn redirect_url(&self, request: FiatBuyRequest, quote: MoonPayBuyQuote) -> String {
        let mut components = Url::parse(MOONPAY_REDIRECT_URL).unwrap();

        components
            .query_pairs_mut()
            .append_pair("apiKey", &self.api_key)
            .append_pair("currencyCode", &quote.quote_currency_code)
            .append_pair(
                "baseCurrencyAmount",
                &request.clone().fiat_amount.to_string(),
            )
            .append_pair("walletAddress", request.wallet_address.as_str());

        let query = components.query().unwrap();
        let signature = self.generate_signature(format!("?{}", &query).as_str());
        components
            .query_pairs_mut()
            .append_pair("signature", &signature);
        components.as_str().to_string()
    }

    fn generate_signature(&self, query: &str) -> String {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query.as_bytes());
        let result = mac.finalize();
        let signature = result.into_bytes();
        general_purpose::STANDARD.encode(signature)
    }
}
