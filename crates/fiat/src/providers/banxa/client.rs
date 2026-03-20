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
        BanxaClient {
            client,
            url,
            merchant_key,
            secret_key,
        }
    }

    pub fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(self.secret_key.as_str()).unwrap());
        headers
    }

    pub async fn get_assets_buy(&self) -> Result<Vec<Asset>, reqwest::Error> {
        self.get_assets_by_order_type(ORDER_TYPE_BUY).await
    }
    async fn get_assets_by_order_type(&self, order_type: &str) -> Result<Vec<Asset>, reqwest::Error> {
        let url = format!("{}/{}/v2/crypto/{}", API_URL, self.merchant_key, order_type);
        self.client.get(&url).headers(self.get_headers()).send().await?.json().await
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, reqwest::Error> {
        let url = format!("{}/{}/v2/orders/{}", API_URL, self.merchant_key, order_id);
        self.client.get(&url).headers(self.get_headers()).send().await?.json().await
    }

    pub async fn get_quote_buy(&self, symbol: &str, chain: &str, fiat_currency: &str, fiat_amount: f64) -> Result<Quote, Box<dyn std::error::Error + Send + Sync>> {
        let fiat_amount_str = fiat_amount.to_string();
        let query = vec![
            ("paymentMethodId", "debit-credit-card"),
            ("crypto", symbol),
            ("blockchain", chain),
            ("fiat", fiat_currency),
            ("fiatAmount", fiat_amount_str.as_str()),
        ];
        let url = format!("{}/{}/v2/quotes/buy", API_URL, self.merchant_key);
        Ok(self.client.get(&url).query(&query).headers(self.get_headers()).send().await?.json().await?)
    }

    pub async fn get_countries(&self) -> Result<Vec<Country>, reqwest::Error> {
        let url = format!("{}/{}/v2/countries", API_URL, self.merchant_key);
        self.client.get(&url).headers(self.get_headers()).send().await?.json().await
    }

    pub async fn get_fiat_currencies(&self, order_type: &str) -> Result<Vec<FiatCurrency>, reqwest::Error> {
        let url = format!("{}/{}/v2/fiats/{}", API_URL, self.merchant_key, order_type);
        self.client.get(&url).headers(self.get_headers()).send().await?.json().await
    }

    pub fn build_quote_url(&self, amount: f64, fiat_currency: &str, symbol: &str, network: &str, wallet_address: &str) -> FiatQuoteUrl {
        let mut components = Url::parse(&self.url).unwrap();
        components
            .query_pairs_mut()
            .append_pair("orderType", ORDER_TYPE_BUY)
            .append_pair("coinType", symbol)
            .append_pair("blockchain", network)
            .append_pair("fiatType", fiat_currency)
            .append_pair("fiatAmount", &amount.to_string())
            .append_pair("walletAddress", wallet_address);

        FiatQuoteUrl {
            redirect_url: components.as_str().to_string(),
        }
    }
}
