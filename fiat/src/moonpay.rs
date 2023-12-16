use crate::model::{FiatClient, FiatMapping};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use primitives::{
    fiat_provider::FiatProviderName, fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest,
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
