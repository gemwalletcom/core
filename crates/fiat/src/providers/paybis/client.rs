use super::models::{Assets, PaybisData, PaybisQuote, PaybisResponse, PaymentMethodWithLimits, QuoteRequest, Request, RequestResponse};
use crate::rsa_signature::generate_rsa_pss_signature;
use primitives::FiatProviderName;
use reqwest::Client;
use url::Url;

const PAYBIS_API_BASE_URL: &str = "https://widget-api.paybis.com";
const PAYBIS_WIDGET_URL: &str = "https://widget.paybis.com";

pub struct PaybisClient {
    client: Client,
    api_key: String,
    private_key: String,
}

impl PaybisClient {
    pub const NAME: FiatProviderName = FiatProviderName::Paybis;

    pub fn new(client: Client, api_key: String, private_key: String) -> Self {
        Self { client, api_key, private_key }
    }

    fn sign_request(&self, body: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        generate_rsa_pss_signature(&self.private_key, body)
    }

    async fn signed_post<T: serde::de::DeserializeOwned>(&self, url: String, body: String) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let signature = self.sign_request(&body)?;
        self.client
            .post(url)
            .header("Authorization", &self.api_key)
            .header("X-Request-Signature", signature)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?
            .json::<PaybisResponse<T>>()
            .await?
            .into()
    }

    pub async fn get_buy_quote(
        &self,
        crypto_currency: String,
        fiat_currency: String,
        fiat_amount: f64,
    ) -> Result<PaybisQuote, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = QuoteRequest {
            amount: fiat_amount.to_string(),
            direction_change: "from".to_string(),
            is_received_amount: false,
            currency_code_from: fiat_currency,
            currency_code_to: crypto_currency,
        };

        let body = serde_json::to_string(&request_body)?;
        let url = format!("{PAYBIS_API_BASE_URL}/v2/quote");
        self.signed_post(url, body).await
    }

    pub async fn get_sell_quote(
        &self,
        crypto_currency: String,
        fiat_currency: String,
        crypto_amount: f64,
    ) -> Result<PaybisQuote, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = QuoteRequest {
            amount: crypto_amount.to_string(),
            direction_change: "from".to_string(),
            is_received_amount: false,
            currency_code_from: crypto_currency,
            currency_code_to: fiat_currency,
        };

        let body = serde_json::to_string(&request_body)?;
        let url = format!("{PAYBIS_API_BASE_URL}/v2/quote");
        self.signed_post(url, body).await
    }

    pub async fn get_assets(&self) -> Result<Assets, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{PAYBIS_API_BASE_URL}/v2/currency/pairs/buy-crypto");
        self.client
            .get(url)
            .header("Authorization", &self.api_key)
            .send()
            .await?
            .json::<PaybisResponse<Assets>>()
            .await?
            .into()
    }

    pub async fn get_payment_method_limits(&self) -> Result<PaybisData<Vec<PaymentMethodWithLimits>>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{PAYBIS_API_BASE_URL}/v2/payment-method-list-with-limits");
        self.client
            .get(url)
            .header("Authorization", &self.api_key)
            .send()
            .await?
            .json::<PaybisResponse<PaybisData<Vec<PaymentMethodWithLimits>>>>()
            .await?
            .into()
    }

    pub async fn create_request(&self, request_body: Request) -> Result<RequestResponse, Box<dyn std::error::Error + Send + Sync>> {
        let body = serde_json::to_string(&request_body)?;
        let url = format!("{PAYBIS_API_BASE_URL}/v3/request");
        self.signed_post(url, body).await
    }

    pub async fn get_redirect_url(
        &self,
        wallet_address: &str,
        from_currency: &str,
        to_currency: &str,
        amount: f64,
        is_buy: bool,
        user_ip: &str,
        locale: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = if is_buy {
            Request::new_buy(
                wallet_address.to_owned(),
                wallet_address.to_owned(),
                to_currency.to_string(),
                from_currency.to_string(),
                amount,
                user_ip.to_string(),
                locale.to_string(),
            )
        } else {
            Request::new_sell(
                wallet_address.to_owned(),
                wallet_address.to_owned(),
                to_currency.to_string(),
                from_currency.to_string(),
                amount,
                user_ip.to_string(),
                locale.to_string(),
            )
        };

        let response = self.create_request(request_body).await?;

        let mut url = Url::parse(PAYBIS_WIDGET_URL)?;
        url.query_pairs_mut().append_pair("requestId", &response.request_id);

        Ok(url.to_string())
    }
}
