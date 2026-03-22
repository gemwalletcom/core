use std::error::Error;

use primitives::FiatProviderName;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};

use super::models::{Asset, CheckoutOrder, Country, CreateOrderRequest, FiatCurrency, Order, PAYMENT_METHOD_CARD, Quote};

const API_URL: &str = "https://api.banxa.com";
const BUY_ORDER_TYPE: &str = "buy";

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
        let url = format!("{API_URL}/{}/v2/crypto/{BUY_ORDER_TYPE}", self.merchant_key);
        Ok(self.client.get(&url).headers(self.headers()?).send().await?.json().await?)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, Box<dyn Error + Send + Sync>> {
        let url = format!("{API_URL}/{}/v2/orders/{order_id}", self.merchant_key);
        Ok(self.client.get(&url).headers(self.headers()?).send().await?.json().await?)
    }

    pub async fn get_quote_buy(&self, symbol: &str, chain: &str, fiat_currency: &str, fiat_amount: f64) -> Result<Quote, Box<dyn Error + Send + Sync>> {
        let fiat_amount = fiat_amount.to_string();
        let query = vec![
            ("paymentMethodId", PAYMENT_METHOD_CARD),
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

    pub async fn get_fiat_currencies_buy(&self) -> Result<Vec<FiatCurrency>, Box<dyn Error + Send + Sync>> {
        let url = format!("{API_URL}/{}/v2/fiats/{BUY_ORDER_TYPE}", self.merchant_key);
        Ok(self.client.get(&url).headers(self.headers()?).send().await?.json().await?)
    }

    pub async fn create_buy_order(
        &self,
        quote_id: String,
        fiat_amount: f64,
        fiat_currency: String,
        symbol: String,
        network: String,
        wallet_address: String,
    ) -> Result<CheckoutOrder, Box<dyn Error + Send + Sync>> {
        let request = CreateOrderRequest::new(quote_id, symbol, fiat_currency, fiat_amount, network, wallet_address, self.url.clone());
        let url = format!("{API_URL}/{}/v2/buy", self.merchant_key);
        let response = self.client.post(&url).headers(self.headers()?).json(&request).send().await?.error_for_status()?;
        response.json().await.map_err(|e| e.into())
    }
}
