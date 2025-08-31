use crate::hmac_signature::generate_hmac_signature_from_base64_key;
use crate::model::FiatProviderAsset;
use crate::providers::paybis::mapper::map_asset_id;

use super::model::{Currency, PaybisAssetsResponse, PaybisQuote, QuoteRequest, PaybisTransactionResponse};
use number_formatter::BigNumberFormatter;
use primitives::{FiatBuyQuote, FiatProviderName, FiatQuote, FiatQuoteType, FiatSellQuote};
use reqwest::Client;
use std::collections::BTreeMap;
use url::Url;

const PAYBIS_API_BASE_URL: &str = "https://widget-api.paybis.com";
const PAYBIS_WIDGET_URL: &str = "https://widget.paybis.com";

pub struct PaybisClient {
    client: Client,
    api_key: String,
    secret_key: String,
}

impl PaybisClient {
    pub const NAME: FiatProviderName = FiatProviderName::Paybis;

    pub fn new(client: Client, api_key: String, secret_key: String) -> Self {
        Self { client, api_key, secret_key }
    }

    pub async fn get_buy_quote(&self, crypto_currency: String, fiat_currency: String, fiat_amount: f64) -> Result<PaybisQuote, reqwest::Error> {
        let request_body = QuoteRequest {
            amount: fiat_amount.to_string(),
            direction_change: "from".to_string(),
            is_received_amount: false,
            currency_code_from: fiat_currency,
            currency_code_to: crypto_currency,
        };
        let url = format!("{PAYBIS_API_BASE_URL}/v2/public/quote");
        self.client
            .post(url)
            .query(&[("apikey", &self.api_key)])
            .json(&request_body)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_sell_quote(&self, crypto_currency: String, fiat_currency: String, crypto_amount: f64) -> Result<PaybisQuote, reqwest::Error> {
        let request_body = QuoteRequest {
            amount: crypto_amount.to_string(),
            direction_change: "from".to_string(),
            is_received_amount: false,
            currency_code_from: crypto_currency,
            currency_code_to: fiat_currency,
        };
        let url = format!("{PAYBIS_API_BASE_URL}/v2/public/quote");
        self.client
            .post(url)
            .query(&[("apikey", &self.api_key)])
            .json(&request_body)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_assets(&self) -> Result<PaybisAssetsResponse, reqwest::Error> {
        let url = format!("{PAYBIS_API_BASE_URL}/v2/public/currency/pairs/buy-crypto");
        self.client.get(url).query(&[("apikey", &self.api_key)]).send().await?.json().await
    }

    pub async fn get_transaction(&self, transaction_id: &str) -> Result<PaybisTransactionResponse, reqwest::Error> {
        let url = format!("{PAYBIS_API_BASE_URL}/v2/transactions");
        self.client
            .get(url)
            .query(&[("apikey", self.api_key.as_str()), ("transaction_id", transaction_id)])
            .send()
            .await?
            .json()
            .await
    }

    pub fn map_asset(currency: Currency) -> Option<FiatProviderAsset> {
        let asset = map_asset_id(currency.clone())?;

        Some(FiatProviderAsset {
            id: currency.code.clone(),
            chain: Some(asset.chain),
            token_id: asset.token_id,
            symbol: currency.code.clone(),
            network: currency.blockchain_name.clone(),
            enabled: true,
            unsupported_countries: Some(currency.unsupported_countries()),
        })
    }

    pub fn get_buy_fiat_quote(&self, request: FiatBuyQuote, quote: PaybisQuote) -> FiatQuote {
        let payment_method = quote.payment_methods.first().unwrap();
        let crypto_amount: f64 = payment_method.amount_to.amount.parse().unwrap_or(0.0);

        let crypto_value = BigNumberFormatter::f64_as_value(crypto_amount, request.asset.decimals as u32).unwrap_or_default();

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency.clone(),
            crypto_amount,
            crypto_value,
            redirect_url: self.redirect_url(
                &request.wallet_address,
                &request.fiat_currency,
                &quote.currency_code_to,
                request.fiat_amount,
                true,
            ),
        }
    }

    pub fn get_sell_fiat_quote(&self, _request: FiatSellQuote, _quote: PaybisQuote) -> FiatQuote {
        unimplemented!();
        // let payment_method = quote.payment_methods.first().unwrap();
        // let fiat_amount: f64 = payment_method.amount_to.amount.parse().unwrap_or(0.0);

        // FiatQuote {
        //     provider: Self::NAME.as_fiat_provider(),
        //     quote_type: FiatQuoteType::Sell,
        //     fiat_amount,
        //     fiat_currency: request.fiat_currency.clone(),
        //     crypto_amount: request.crypto_amount,
        //     crypto_value: request.crypto_value,
        //     redirect_url: self.redirect_url(
        //         &request.wallet_address,
        //         &quote.currency_code_to,
        //         &request.fiat_currency,
        //         request.crypto_amount,
        //         false,
        //     ),
        // }
    }

    pub fn redirect_url(&self, wallet_address: &str, from_currency: &str, to_currency: &str, amount: f64, is_buy: bool) -> String {
        let mut url = Url::parse(PAYBIS_WIDGET_URL).unwrap();

        // Use BTreeMap to ensure parameters are always sorted alphabetically
        let mut params = BTreeMap::new();
        params.insert("currencyCodeFrom", from_currency.to_string());
        params.insert("currencyCodeTo", to_currency.to_string());
        params.insert("partnerId", self.api_key.clone());
        params.insert("amountFrom", amount.to_string());
        params.insert("cryptoAddress", wallet_address.to_string());

        if is_buy {
            params.insert("transactionFlow", "buyCrypto".to_string());
        } else {
            params.insert("transactionFlow", "sellCrypto".to_string());
        }

        for (key, value) in params {
            url.query_pairs_mut().append_pair(key, &value);
        }

        self.sign(url).expect("Failed to sign URL")
    }

    fn sign(&self, mut url: Url) -> Option<String> {
        let query = url.query()?;
        let query = format!("?{}", &query);

        let signature = generate_hmac_signature_from_base64_key(&self.secret_key, &query)?;
        url.query_pairs_mut().append_pair("signature", &signature);
        Some(url.as_str().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redirect_url_buy() {
        let client = Client::new();
        // Use a base64-encoded test secret key
        let base64_secret = "dGVzdF9zZWNyZXRfa2V5"; // "test_secret_key" in base64
        let paybis_client = PaybisClient::new(client, "test_api_key".to_string(), base64_secret.to_string());
        let redirect_url = paybis_client.redirect_url("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq", "USD", "BTC", 100.0, true);

        // The expected signature for this exact query string with the test secret
        assert_eq!(
            redirect_url,
            "https://widget.paybis.com/?amountFrom=100&cryptoAddress=bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq&currencyCodeFrom=USD&currencyCodeTo=BTC&partnerId=test_api_key&transactionFlow=buyCrypto&signature=7ul26u1ze6T78AdtMPqHv64EK8eyZbfGXCCbjq%2F5gH0%3D"
        );
    }

    #[test]
    fn test_redirect_url_sell() {
        let client = Client::new();
        // Use a base64-encoded test secret key
        let base64_secret = "dGVzdF9zZWNyZXRfa2V5"; // "test_secret_key" in base64
        let paybis_client = PaybisClient::new(client, "test_api_key".to_string(), base64_secret.to_string());
        let redirect_url = paybis_client.redirect_url("0x742d35Cc6634C0532925a3b844Bc9e7595f5843", "ETH", "EUR", 2.5, false);

        // The expected signature for this exact query string with the test secret
        assert_eq!(
            redirect_url,
            "https://widget.paybis.com/?amountFrom=2.5&cryptoAddress=0x742d35Cc6634C0532925a3b844Bc9e7595f5843&currencyCodeFrom=ETH&currencyCodeTo=EUR&partnerId=test_api_key&transactionFlow=sellCrypto&signature=r9JL0lr%2BE%2BaS6OOWpFiz8Q099rXAh41t48kf4mieinA%3D"
        );
    }
}
