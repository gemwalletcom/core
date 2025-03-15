use std::error::Error;

use crate::error::FiatError;
use crate::model::{filter_token_id, FiatProviderAsset};

use super::mapper::map_asset_chain;
use super::model::{Asset, MoonPayBuyQuote, MoonPayIpAddress, MoonPaySellQuote};
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use primitives::{FiatQuoteRequest, FiatProviderName, FiatQuote, FiatTransactionType};
use reqwest::Client;
use sha2::Sha256;
use url::Url;

pub struct MoonPayClient {
    client: Client,
    api_key: String,
    secret_key: String,
}

const MOONPAY_API_BASE_URL: &str = "https://api.moonpay.com";
const MOONPAY_BUY_REDIRECT_URL: &str = "https://buy.moonpay.com";
const MOONPAY_SELL_REDIRECT_URL: &str = "https://sell.moonpay.com";

impl MoonPayClient {
    pub const NAME: FiatProviderName = FiatProviderName::MoonPay;

    pub fn new(client: Client, api_key: String, secret_key: String) -> Self {
        Self { client, api_key, secret_key }
    }

    pub async fn get_ip_address(&self, ip_address: &str) -> Result<MoonPayIpAddress, reqwest::Error> {
        let url = format!("{}/v4/ip_address/?ipAddress={}&apiKey={}", MOONPAY_API_BASE_URL, ip_address, self.api_key,);

        let response = self.client.get(&url).send().await?;
        let ip_address_result = response.json::<MoonPayIpAddress>().await?;

        Ok(ip_address_result)
    }

    pub async fn get_buy_quote(&self, symbol: String, fiat_currency: String, fiat_amount: f64) -> Result<MoonPayBuyQuote, reqwest::Error> {
        let url = format!(
            "{}/v3/currencies/{}/buy_quote/?baseCurrencyCode={}&baseCurrencyAmount={}&areFeesIncluded={}&apiKey={}",
            MOONPAY_API_BASE_URL, symbol, fiat_currency, fiat_amount, "true", self.api_key,
        );

        self.client.get(&url).send().await?.json::<MoonPayBuyQuote>().await
    }

    pub async fn get_sell_quote(&self, symbol: String, fiat_currency: String, crypto_amount: f64) -> Result<MoonPaySellQuote, reqwest::Error> {
        let url = format!(
            "{}/v3/currencies/{}/sell_quote/?quoteCurrencyCode={}&baseCurrencyAmount={}&areFeesIncluded={}&apiKey={}",
            MOONPAY_API_BASE_URL, symbol, fiat_currency, crypto_amount, "true", self.api_key,
        );

        self.client.get(&url).send().await?.json::<MoonPaySellQuote>().await
    }

    pub async fn get_assets(&self) -> Result<Vec<Asset>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/v3/currencies", MOONPAY_API_BASE_URL);
        let assets = self.client.get(&url).send().await?.json::<Vec<Asset>>().await?;
        Ok(assets)
    }

    pub async fn validate_quote(&self, quote: &MoonPayBuyQuote, ip_address: MoonPayIpAddress) -> Result<(), Box<dyn Error + Send + Sync>> {
        if quote.quote_currency.not_allowed_countries.contains(&ip_address.alpha2) {
            return Err(FiatError::UnsupportedCountry(ip_address.alpha2).into());
        }

        if &ip_address.state == "US" && quote.quote_currency.not_allowed_us_states.contains(&ip_address.state) {
            return Err(FiatError::UnsupportedState(ip_address.state).into());
        }
        Ok(())
    }

    pub async fn get_transactions(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
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
        let chain = map_asset_chain(asset.clone());
        let token_id = filter_token_id(chain, asset.clone().metadata?.contract_address);
        let enabled = !asset.is_suspended.unwrap_or(true);
        Some(FiatProviderAsset {
            id: asset.clone().code,
            chain,
            token_id,
            symbol: asset.code,
            network: asset.metadata.map(|x| x.network_code),
            enabled,
        })
    }

    pub fn get_buy_fiat_quote(&self, request: FiatQuoteRequest, quote: MoonPayBuyQuote) -> FiatQuote {
        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatTransactionType::Buy,
            fiat_amount: request.clone().fiat_amount,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount: quote.quote_currency_amount,
            redirect_url: self.redirect_buy_url(request.fiat_amount, &request.wallet_address, &quote.quote_currency_code),
        }
    }

    pub fn get_sell_fiat_quote(&self, request: FiatQuoteRequest, quote: MoonPaySellQuote) -> FiatQuote {
        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatTransactionType::Sell,
            fiat_amount: quote.quote_currency_amount,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount: quote.base_currency_amount,
            redirect_url: self.redirect_sell_url(request.crypto_amount.unwrap_or_default(), &request.wallet_address, &quote.base_currency_code),
        }
    }

    // docs: https://dev.moonpay.com/docs/ramps-sdk-buy-params
    pub fn redirect_buy_url(&self, amount: f64, address: &str, symbol: &str) -> String {
        let mut components = Url::parse(MOONPAY_BUY_REDIRECT_URL).unwrap();
        components
            .query_pairs_mut()
            .append_pair("apiKey", &self.api_key)
            .append_pair("currencyCode", symbol)
            .append_pair("baseCurrencyAmount", &amount.to_string())
            .append_pair("walletAddress", address);
        self.sign(components)
    }

    // docs: https://dev.moonpay.com/docs/ramps-sdk-sell-params
    pub fn redirect_sell_url(&self, amount: f64, address: &str, symbol: &str) -> String {
        let mut components = Url::parse(MOONPAY_SELL_REDIRECT_URL).unwrap();
        components
            .query_pairs_mut()
            .append_pair("apiKey", &self.api_key)
            .append_pair("defaultBaseCurrencyCode", symbol)
            .append_pair("baseCurrencyAmount", &amount.to_string())
            .append_pair("refundWalletAddress", address);
        self.sign(components)
    }

    fn sign(&self, mut components: Url) -> String {
        let query = components.query().unwrap();
        let signature = self.generate_signature(format!("?{}", &query).as_str());
        components.query_pairs_mut().append_pair("signature", &signature);
        components.as_str().to_string()
    }

    fn generate_signature(&self, query: &str) -> String {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(query.as_bytes());
        let result = mac.finalize();
        let signature = result.into_bytes();
        general_purpose::STANDARD.encode(signature)
    }
}
