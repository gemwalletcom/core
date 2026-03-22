use std::error::Error;

use primitives::{FiatProviderName, FiatQuoteUrl};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use url::Url;

use super::models::{Asset, Country, FiatCurrency, ORDER_TYPE_BUY, Order, Quote};

const API_URL: &str = "https://api.banxa.com";

pub struct BanxaClient {
    pub client: Client,
    pub url: String,
    pub merchant_key: String,
    pub secret_key: String,
}

impl BanxaClient {
    pub const NAME: FiatProviderName = FiatProviderName::Banxa;

    pub fn new(client: Client, url: String, merchant_key: String, secret_key: String) -> Self {
        Self {
            client,
            url,
            merchant_key,
            secret_key,
        }
    }

    fn headers(&self) -> Result<HeaderMap, Box<dyn Error + Send + Sync>> {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(self.secret_key.as_str())?);
        Ok(headers)
    }

    pub async fn get_assets_buy(&self) -> Result<Vec<Asset>, Box<dyn Error + Send + Sync>> {
        self.get_assets_by_order_type(ORDER_TYPE_BUY).await
    }

    async fn get_assets_by_order_type(&self, order_type: &str) -> Result<Vec<Asset>, Box<dyn Error + Send + Sync>> {
        let url = format!("{API_URL}/{}/v2/crypto/{order_type}", self.merchant_key);
        Ok(self.client.get(&url).headers(self.headers()?).send().await?.json().await?)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, Box<dyn Error + Send + Sync>> {
        let url = format!("{API_URL}/{}/v2/orders/{order_id}", self.merchant_key);
        Ok(self.client.get(&url).headers(self.headers()?).send().await?.json().await?)
    }

    pub async fn get_quote_buy(&self, symbol: &str, chain: &str, fiat_currency: &str, fiat_amount: f64) -> Result<Quote, Box<dyn Error + Send + Sync>> {
        let fiat_amount = fiat_amount.to_string();
        let query = vec![
            ("paymentMethodId", "debit-credit-card"),
            ("crypto", symbol),
            ("blockchain", chain),
            ("fiat", fiat_currency),
            ("fiatAmount", fiat_amount.as_str()),
        ];
        let url = format!("{API_URL}/{}/v2/quotes/buy", self.merchant_key);
        Ok(self.client.get(&url).query(&query).headers(self.headers()?).send().await?.json().await?)
    }

    pub async fn get_countries(&self) -> Result<Vec<Country>, Box<dyn Error + Send + Sync>> {
        let url = format!("{API_URL}/{}/v2/countries", self.merchant_key);
        Ok(self.client.get(&url).headers(self.headers()?).send().await?.json().await?)
    }

    pub async fn get_fiat_currencies(&self, order_type: &str) -> Result<Vec<FiatCurrency>, Box<dyn Error + Send + Sync>> {
        let url = format!("{API_URL}/{}/v2/fiats/{order_type}", self.merchant_key);
        Ok(self.client.get(&url).headers(self.headers()?).send().await?.json().await?)
    }

    pub fn build_quote_url(&self, amount: f64, fiat_currency: &str, symbol: &str, network: &str, wallet_address: &str) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        let mut url = Url::parse(&self.url)?;
        url.query_pairs_mut()
            .append_pair("orderType", ORDER_TYPE_BUY)
            .append_pair("coinType", symbol)
            .append_pair("blockchain", network)
            .append_pair("fiatType", fiat_currency)
            .append_pair("fiatAmount", &amount.to_string())
            .append_pair("walletAddress", wallet_address);

        Ok(FiatQuoteUrl {
            redirect_url: url.to_string(),
            provider_transaction_id: None,
        })
    }
}
