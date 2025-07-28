use crate::hmac_signature::generate_hmac_signature;
use crate::model::FiatProviderAsset;
use crate::providers::paybis::mapper::map_asset_id;

use super::model::{Currency, PaybisAssetsResponse, PaybisQuote, QuoteRequest};
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

    pub fn get_sell_fiat_quote(&self, request: FiatSellQuote, quote: PaybisQuote) -> FiatQuote {
        let payment_method = quote.payment_methods.first().unwrap();
        let fiat_amount: f64 = payment_method.amount_to.amount.parse().unwrap_or(0.0);

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Sell,
            fiat_amount,
            fiat_currency: request.fiat_currency.clone(),
            crypto_amount: request.crypto_amount,
            crypto_value: request.crypto_value,
            redirect_url: self.redirect_url(
                &request.wallet_address,
                &quote.currency_code_to,
                &request.fiat_currency,
                request.crypto_amount,
                false,
            ),
        }
    }

    pub fn redirect_url(&self, wallet_address: &str, from_currency: &str, to_currency: &str, amount: f64, is_buy: bool) -> String {
        let mut url = Url::parse(PAYBIS_WIDGET_URL).unwrap();

        // Use BTreeMap to ensure parameters are always sorted alphabetically
        let mut params = BTreeMap::new();
        params.insert("currencyCodeFrom", from_currency.to_string());
        params.insert("currencyCodeTo", to_currency.to_string());
        params.insert("partnerId", self.api_key.clone());
        params.insert("requestedAmount", amount.to_string());
        params.insert("requestedAmountType", "from".to_string());
        params.insert("walletAddress", wallet_address.to_string());

        if is_buy {
            params.insert("flow", "buyCrypto".to_string());
        } else {
            params.insert("flow", "sellCrypto".to_string());
        }

        // Add all parameters to URL in sorted order
        for (key, value) in params {
            url.query_pairs_mut().append_pair(key, &value);
        }

        self.sign(url).expect("Failed to sign URL")
    }

    fn sign(&self, mut url: Url) -> Option<String> {
        let query = url.query()?;
        let signature = generate_hmac_signature(&self.secret_key, query);
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
        let paybis_client = PaybisClient::new(client, "test_api_key".to_string(), "test_secret_key".to_string());
        let redirect_url = paybis_client.redirect_url("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq", "USD", "BTC", 100.0, true);
        let url = Url::parse(&redirect_url).unwrap();

        assert_eq!(url.as_str(), "https://widget.paybis.com/?currencyCodeFrom=USD&currencyCodeTo=BTC&flow=buyCrypto&partnerId=test_api_key&requestedAmount=100&requestedAmountType=from&walletAddress=bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq&signature=X7gyrLY3co8KBGUl8%2FM6vKtY%2F%2B7n8Hb99Smg%2F725vyw%3D");
    }

    #[test]
    fn test_redirect_url_sell() {
        let client = Client::new();
        let paybis_client = PaybisClient::new(client, "test_api_key".to_string(), "test_secret_key".to_string());
        let redirect_url = paybis_client.redirect_url("0x742d35Cc6634C0532925a3b844Bc9e7595f5843", "ETH", "EUR", 2.5, false);

        let url = Url::parse(&redirect_url).unwrap();
        assert_eq!(url.as_str(), "https://widget.paybis.com/?currencyCodeFrom=ETH&currencyCodeTo=EUR&flow=sellCrypto&partnerId=test_api_key&requestedAmount=2.5&requestedAmountType=from&walletAddress=0x742d35Cc6634C0532925a3b844Bc9e7595f5843&signature=uKyvDcC8D29AIRFfQALLhjDpXsIkqiahqbEhf9O86LA%3D");
    }
}
