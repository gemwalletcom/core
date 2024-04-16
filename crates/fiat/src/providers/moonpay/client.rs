use crate::model::{filter_token_id, FiatProviderAsset};

use super::model::{Asset, MoonPayBuyQuote, MoonPayIpAddress};
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use primitives::{
    fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest, Chain, FiatProviderName,
};
use reqwest::Client;
use sha2::Sha256;
use url::Url;

pub struct MoonPayClient {
    client: Client,
    api_key: String,
    secret_key: String,
}

const MOONPAY_API_BASE_URL: &str = "https://api.moonpay.com";
const MOONPAY_REDIRECT_URL: &str = "https://buy.moonpay.com";

impl MoonPayClient {
    pub const NAME: FiatProviderName = FiatProviderName::MoonPay;

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

    pub async fn get_buy_quote(
        &self,
        symbol: String,
        fiat_currency: String,
        fiat_amount: f64,
    ) -> Result<MoonPayBuyQuote, reqwest::Error> {
        let url = format!(
            "{}/v3/currencies/{}/buy_quote/?baseCurrencyCode={}&baseCurrencyAmount={}&areFeesIncluded={}&apiKey={}",
            MOONPAY_API_BASE_URL,
            symbol,
            fiat_currency,
            fiat_amount,
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
        Ok(quote)
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

    pub async fn get_transactions(
        &self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        // let url = format!("{}/v1/transactions", MOONPAY_API_BASE_URL);
        // let assets = self
        //     .client
        //     .get(&url)
        //     .send()
        //     .await?
        //     .json::<Vec<Asset>>()
        //     .await?;
        // Ok(assets)
        Ok(vec![])
    }

    pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
        let chain = Self::map_asset_chain(asset.clone());
        let token_id = filter_token_id(asset.clone().metadata?.contract_address);
        let enabled = !asset.is_suspended.unwrap_or(true);
        Some(FiatProviderAsset {
            id: asset.clone().code,
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
            "near" => Some(Chain::Near),
            "linea" => Some(Chain::Linea),
            "zksync" => Some(Chain::ZkSync),
            "celo" => Some(Chain::Celo),
            _ => None,
        }
    }

    pub fn get_fiat_quote(&self, request: FiatBuyRequest, quote: MoonPayBuyQuote) -> FiatQuote {
        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
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
