use crate::model::FiatMapping;
use number_formatter::BigNumberFormatter;
use primitives::{FiatBuyQuote, FiatProviderName, FiatQuote, FiatQuoteType, FiatSellQuote};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use url::Url;

use super::models::{Asset, Country, FiatCurrency, ORDER_TYPE_BUY, ORDER_TYPE_SELL, Order, PaymentMethod, Quote};

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
    pub async fn get_assets_sell(&self) -> Result<Vec<Asset>, reqwest::Error> {
        self.get_assets_by_order_type(ORDER_TYPE_SELL).await
    }

    async fn get_assets_by_order_type(&self, order_type: &str) -> Result<Vec<Asset>, reqwest::Error> {
        let url = format!("{}/{}/v2/crypto/{}", API_URL, self.merchant_key, order_type);
        self.client.get(&url).headers(self.get_headers()).send().await?.json().await
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, reqwest::Error> {
        let url = format!("{}/{}/v2/orders/{}", API_URL, self.merchant_key, order_id);
        self.client.get(&url).headers(self.get_headers()).send().await?.json().await
    }

    pub async fn get_quote_buy(
        &self,
        symbol: &str,
        chain: &str,
        fiat_currency: &str,
        fiat_amount: f64,
    ) -> Result<Quote, Box<dyn std::error::Error + Send + Sync>> {
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

    pub async fn get_quote_sell(&self, method: &str, symbol: &str, chain: &str, fiat_currency: &str, crypto_amount: f64) -> Result<Quote, reqwest::Error> {
        let crypto_amount_str = crypto_amount.to_string();
        let query = vec![
            ("paymentMethodId", method),
            ("crypto", symbol),
            ("blockchain", chain),
            ("fiat", fiat_currency),
            ("cryptoAmount", crypto_amount_str.as_str()),
        ];
        let url = format!("{}/{}/v2/quotes/sell", API_URL, self.merchant_key);
        self.client.get(&url).query(&query).headers(self.get_headers()).send().await?.json().await
    }

    pub async fn get_payment_methods(&self, order_type: &str) -> Result<Vec<PaymentMethod>, reqwest::Error> {
        let url = format!("{}/{}/v2/payment-methods/{}", API_URL, self.merchant_key, order_type);
        self.client.get(&url).headers(self.get_headers()).send().await?.json().await
    }

    pub async fn get_countries(&self) -> Result<Vec<Country>, reqwest::Error> {
        let url = format!("{}/{}/v2/countries", API_URL, self.merchant_key);
        self.client.get(&url).headers(self.get_headers()).send().await?.json().await
    }

    pub async fn get_fiat_currencies(&self, order_type: &str) -> Result<Vec<FiatCurrency>, reqwest::Error> {
        let url = format!("{}/{}/v2/fiats/{}", API_URL, self.merchant_key, order_type);
        self.client.get(&url).headers(self.get_headers()).send().await?.json().await
    }

    pub fn get_fiat_buy_quote(&self, request: FiatBuyQuote, fiat_mapping: FiatMapping, quote: Quote) -> FiatQuote {
        let redirect_url = self.get_redirect_buy_url(request.clone(), fiat_mapping);
        let crypto_value = BigNumberFormatter::f64_as_value(quote.crypto_amount, request.asset.decimals as u32).unwrap_or_default();

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency.as_ref().to_string(),
            crypto_amount: quote.crypto_amount,
            crypto_value,
            redirect_url,
        }
    }

    pub fn get_fiat_sell_quote(&self, request: FiatSellQuote, fiat_mapping: FiatMapping, quote: Quote) -> FiatQuote {
        let redirect_url = self.get_redirect_sell_url(request.clone(), fiat_mapping);

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Sell,
            fiat_amount: quote.fiat_amount,
            fiat_currency: request.fiat_currency.as_ref().to_string(),
            crypto_amount: request.crypto_amount,
            crypto_value: request.crypto_value,
            redirect_url,
        }
    }

    // URL Parametization https://docs.banxa.com/docs/referral-link

    pub fn get_redirect_buy_url(&self, request: FiatBuyQuote, fiat_mapping: FiatMapping) -> String {
        let mut components = Url::parse(&self.url).unwrap();
        components
            .query_pairs_mut()
            .append_pair("orderType", "buy")
            .append_pair("coinType", &fiat_mapping.asset_symbol.symbol)
            .append_pair("blockchain", &fiat_mapping.asset_symbol.network.unwrap_or_default())
            .append_pair("fiatType", request.fiat_currency.as_ref())
            .append_pair("fiatAmount", &request.fiat_amount.to_string())
            .append_pair("walletAddress", &request.wallet_address);
        components.as_str().to_string()
    }

    pub fn get_redirect_sell_url(&self, request: FiatSellQuote, fiat_mapping: FiatMapping) -> String {
        let mut components = Url::parse(&self.url).unwrap();
        components
            .query_pairs_mut()
            .append_pair("orderType", "sell")
            .append_pair("coinType", &fiat_mapping.asset_symbol.symbol)
            .append_pair("blockchain", &fiat_mapping.asset_symbol.network.unwrap_or_default())
            .append_pair("fiatType", request.fiat_currency.as_ref())
            .append_pair("coinAmount", request.crypto_amount.to_string().as_str())
            .append_pair("walletAddress", &request.wallet_address);
        components.as_str().to_string()
    }
}
