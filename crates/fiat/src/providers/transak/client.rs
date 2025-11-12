use super::models::{
    Asset, CachedToken, Country, CreateWidgetUrlRequest, CreateWidgetUrlResponse, Data, FiatCurrency, Response, TokenResponse, TransakOrderResponse,
    TransakQuote,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as BASE64};
use number_formatter::BigNumberFormatter;
use primitives::FiatBuyQuote;
use primitives::{FiatProviderName, FiatQuote, FiatQuoteType};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const TRANSAK_API_URL: &str = "https://api.transak.com";
const TRANSAK_API_GATEWAY_URL: &str = "https://api-gateway.transak.com";
const TOKEN_TTL_SECONDS: u64 = 3600;

#[derive(Debug, Clone)]
pub struct TransakClient {
    pub client: Client,
    pub api_key: String,
    pub api_secret: String,
    pub referrer_domain: String,
    cached_token: Arc<Mutex<Option<CachedToken>>>,
}

impl TransakClient {
    pub const NAME: FiatProviderName = FiatProviderName::Transak;

    pub fn new(client: Client, api_key: String, api_secret: String, referrer_domain: String) -> Self {
        TransakClient {
            client,
            api_key,
            api_secret,
            referrer_domain,
            cached_token: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_buy_quote(
        &self,
        symbol: String,
        fiat_currency: String,
        fiat_amount: f64,
        network: String,
        ip_address: String,
    ) -> Result<TransakQuote, reqwest::Error> {
        self.get_quote("buy", symbol, fiat_currency, Some(fiat_amount), None, network, ip_address).await
    }

    pub async fn get_quote(
        &self,
        quote_type: &str,
        symbol: String,
        fiat_currency: String,
        fiat_amount: Option<f64>,
        crypto_amount: Option<&str>,
        network: String,
        country_code: String,
    ) -> Result<TransakQuote, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/api/v1/pricing/public/quotes");
        let mut query = vec![
            ("isBuyOrSell", quote_type.to_string()),
            ("quoteCountryCode", country_code.to_string()),
            ("fiatCurrency", fiat_currency.to_string()),
            ("cryptoCurrency", symbol.to_string()),
            ("network", network.to_string()),
            ("partnerApiKey", self.api_key.to_string()),
        ];
        if let Some(amount) = fiat_amount {
            query.push(("fiatAmount", amount.to_string()));
        }
        if let Some(amount) = crypto_amount {
            query.push(("cryptoAmount", amount.to_string()));
        }

        Ok(self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .json::<Response<TransakQuote>>()
            .await?
            .response)
    }

    pub async fn get_fiat_quote_with_redirect(
        &self,
        request: FiatBuyQuote,
        symbol: String,
        fiat_currency: String,
        fiat_amount: f64,
        network: String,
        ip_address: String,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self.get_buy_quote(symbol, fiat_currency, fiat_amount, network, ip_address).await?;

        let crypto_value = BigNumberFormatter::f64_as_value(quote.crypto_amount, request.asset.decimals as u32).ok_or_else(|| {
            format!(
                "Failed to convert crypto amount {} with decimals {}",
                quote.crypto_amount, request.asset.decimals
            )
        })?;
        let redirect_url = self.redirect_url(quote.clone(), request.wallet_address).await?;

        Ok(FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency.as_ref().to_string(),
            crypto_amount: quote.crypto_amount,
            crypto_value,
            redirect_url,
        })
    }

    pub async fn create_widget_url(&self, params: HashMap<String, String>) -> Result<String, reqwest::Error> {
        let access_token = self.get_access_token().await?;
        let url = format!("{TRANSAK_API_GATEWAY_URL}/api/v2/auth/session");

        let request_body = CreateWidgetUrlRequest { params };

        let response: Data<CreateWidgetUrlResponse> = self
            .client
            .post(&url)
            .header("access-token", &access_token)
            .json(&request_body)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.data.widget_url)
    }

    pub async fn redirect_url(&self, quote: TransakQuote, address: String) -> Result<String, reqwest::Error> {
        let mut params = HashMap::new();
        params.insert("apiKey".to_string(), self.api_key.clone());
        params.insert("referrerDomain".to_string(), self.referrer_domain.clone());
        params.insert("fiatAmount".to_string(), quote.fiat_amount.to_string());
        params.insert("fiatCurrency".to_string(), quote.fiat_currency);
        params.insert("cryptoCurrencyCode".to_string(), quote.crypto_currency);
        params.insert("network".to_string(), quote.network);
        params.insert("disableWalletAddressForm".to_string(), "true".to_string());
        params.insert("walletAddress".to_string(), address);

        self.create_widget_url(params).await
    }

    pub async fn get_supported_assets(&self) -> Result<Response<Vec<Asset>>, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/api/v2/currencies/crypto-currencies");
        self.client.get(&url).send().await?.json().await
    }

    pub async fn get_countries(&self) -> Result<Response<Vec<Country>>, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/api/v2/countries");
        self.client.get(&url).send().await?.json().await
    }

    pub async fn get_fiat_currencies(&self) -> Result<Response<Vec<FiatCurrency>>, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/fiat/public/v1/currencies/fiat-currencies");
        self.client.get(&url).send().await?.json().await
    }

    async fn get_access_token(&self) -> Result<String, reqwest::Error> {
        let mut token_guard = self.cached_token.lock().await;

        if let Some(cached) = token_guard.as_ref()
            && cached.is_valid() {
                return Ok(cached.access_token.clone());
            }

        let access_token = self.refresh_token_internal().await?;
        let cached = CachedToken::new(access_token.clone(), TOKEN_TTL_SECONDS);
        *token_guard = Some(cached);

        Ok(access_token)
    }

    async fn refresh_token_internal(&self) -> Result<String, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/partners/api/v2/refresh-token?apiKey={}", self.api_key);
        let body = serde_json::json!({
            "apiKey": self.api_key
        });

        let response: Data<TokenResponse> = self
            .client
            .post(&url)
            .header("api-secret", &self.api_secret)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.data.access_token)
    }

    pub async fn get_transaction(&self, order_id: &str) -> Result<TransakOrderResponse, reqwest::Error> {
        let access_token = self.get_access_token().await?;
        let url = format!("{TRANSAK_API_URL}/partners/api/v2/order/{order_id}");
        let response: Data<TransakOrderResponse> = self.client.get(&url).header("access-token", &access_token).send().await?.json().await?;
        Ok(response.data)
    }

    pub fn decode_jwt_content(&self, jwt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".to_string().into());
        }
        let payload = parts[1];
        let payload = BASE64.decode(payload)?;
        Ok(String::from_utf8(payload)?)
    }
}
