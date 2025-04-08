use crate::model::{filter_token_id, FiatProviderAsset};

use super::mapper::map_asset_chain;
use super::model::{Asset, Country, MoonPayBuyQuote, MoonPayIpAddress, MoonPaySellQuote};
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use number_formatter::BigNumberFormatter;
use primitives::{FiatBuyQuote, FiatProviderName, FiatQuote, FiatQuoteType, FiatQuoteTypeResult, FiatSellQuote};
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
        self.client
            .get(format!("{}/v4/ip_address/", MOONPAY_API_BASE_URL))
            .query(&[("ipAddress", ip_address), ("apiKey", &self.api_key)])
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_buy_quote(&self, symbol: String, fiat_currency: String, fiat_amount: f64) -> Result<MoonPayBuyQuote, reqwest::Error> {
        self.client
            .get(format!("{}/v3/currencies/{}/buy_quote/", MOONPAY_API_BASE_URL, symbol))
            .query(&[
                ("baseCurrencyCode", fiat_currency),
                ("baseCurrencyAmount", fiat_amount.to_string()),
                ("areFeesIncluded", "true".to_string()),
                ("apiKey", self.api_key.clone()),
            ])
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_sell_quote(&self, symbol: String, fiat_currency: String, crypto_amount: f64) -> Result<MoonPaySellQuote, reqwest::Error> {
        self.client
            .get(format!("{}/v3/currencies/{}/sell_quote/", MOONPAY_API_BASE_URL, symbol))
            .query(&[
                ("quoteCurrencyCode", fiat_currency),
                ("baseCurrencyAmount", crypto_amount.to_string()),
                ("areFeesIncluded", "true".to_string()),
                ("apiKey", self.api_key.clone()),
            ])
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_assets(&self) -> Result<Vec<Asset>, reqwest::Error> {
        self.client.get(format!("{}/v3/currencies", MOONPAY_API_BASE_URL)).send().await?.json().await
    }

    pub async fn get_countries(&self) -> Result<Vec<Country>, reqwest::Error> {
        self.client.get(format!("{}/v3/countries", MOONPAY_API_BASE_URL)).send().await?.json().await
    }

    pub async fn get_transactions(&self) -> Result<Vec<String>, reqwest::Error> {
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
        let contract_address = match asset.metadata.as_ref().map(|m| m.network_code.as_str()) {
            Some("ripple") => asset
                .metadata
                .as_ref()
                .and_then(|m| m.contract_address.as_deref().and_then(|s| s.split('.').next_back().map(String::from))),
            // Add other blockchain specific rules here
            _ => asset.clone().metadata?.contract_address,
        };

        let token_id = filter_token_id(chain, contract_address);
        let enabled = !asset.is_suspended.unwrap_or(true);

        Some(FiatProviderAsset {
            id: asset.clone().code,
            chain,
            token_id,
            symbol: asset.clone().code,
            network: asset.clone().metadata.map(|x| x.network_code),
            enabled,
            unsupported_countries: Some(asset.unsupported_countries()),
        })
    }

    pub fn get_buy_fiat_quote(&self, request: FiatBuyQuote, quote: MoonPayBuyQuote) -> FiatQuote {
        let crypto_value = BigNumberFormatter::f64_as_value(quote.quote_currency_amount, quote.quote_currency.decimals).unwrap_or_default();

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.clone().fiat_amount,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount: quote.quote_currency_amount,
            crypto_value,
            redirect_url: self.redirect_url(FiatQuoteTypeResult::Buy(request.clone()), request.fiat_amount, &quote.quote_currency_code),
        }
    }

    pub fn get_sell_fiat_quote(&self, request: FiatSellQuote, quote: MoonPaySellQuote) -> FiatQuote {
        let crypto_value = request.clone().crypto_value;
        let crypto_amount = quote.base_currency_amount;
        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Sell,
            fiat_amount: quote.quote_currency_amount,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount,
            crypto_value,
            redirect_url: self.redirect_url(FiatQuoteTypeResult::Sell(request), crypto_amount, &quote.base_currency_code),
        }
    }

    // docs: https://dev.moonpay.com/docs/ramps-sdk-buy-params
    // docs: https://dev.moonpay.com/docs/ramps-sdk-sell-params
    pub fn redirect_url(&self, quote_type: FiatQuoteTypeResult, amount: f64, symbol: &str) -> String {
        let url = match quote_type {
            FiatQuoteTypeResult::Buy(_) => MOONPAY_BUY_REDIRECT_URL,
            FiatQuoteTypeResult::Sell(_) => MOONPAY_SELL_REDIRECT_URL,
        };
        let mut components = Url::parse(url).unwrap();
        components
            .query_pairs_mut()
            .append_pair("apiKey", &self.api_key)
            .append_pair("baseCurrencyAmount", &amount.to_string());

        match quote_type {
            FiatQuoteTypeResult::Buy(_) => components
                .query_pairs_mut()
                .append_pair("currencyCode", symbol)
                .append_pair("walletAddress", &quote_type.get_wallet_address()),
            FiatQuoteTypeResult::Sell(_) => components
                .query_pairs_mut()
                .append_pair("defaultBaseCurrencyCode", symbol)
                .append_pair("refundWalletAddress", &quote_type.get_wallet_address()),
        };
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
