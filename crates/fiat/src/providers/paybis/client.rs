use super::models::{Assets, PaybisData, PaybisQuote, PaybisResponse, PaymentMethodWithLimits, QuoteRequest, Request, RequestResponse};
use crate::rsa_signature::generate_rsa_pss_signature;
use number_formatter::BigNumberFormatter;
use primitives::{FiatBuyQuote, FiatProviderName, FiatQuote, FiatQuoteType, FiatSellQuote};
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
        let url = format!("{PAYBIS_API_BASE_URL}/v2/public/quote");
        let response: PaybisResponse<PaybisQuote> = self
            .client
            .post(url)
            .query(&[("apikey", &self.api_key)])
            .json(&request_body)
            .send()
            .await?
            .json()
            .await?;

        match response {
            PaybisResponse::Success(quote) => Ok(quote),
            PaybisResponse::Error(error) => Err(error.into_error()),
        }
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
        let url = format!("{PAYBIS_API_BASE_URL}/v2/public/quote");
        let response: PaybisResponse<PaybisQuote> = self
            .client
            .post(url)
            .query(&[("apikey", &self.api_key)])
            .json(&request_body)
            .send()
            .await?
            .json()
            .await?;

        match response {
            PaybisResponse::Success(quote) => Ok(quote),
            PaybisResponse::Error(error) => Err(error.into_error()),
        }
    }

    pub async fn get_assets(&self) -> Result<Assets, reqwest::Error> {
        let url = format!("{PAYBIS_API_BASE_URL}/v2/public/currency/pairs/buy-crypto");
        self.client.get(url).query(&[("apikey", &self.api_key)]).send().await?.json().await
    }

    pub async fn get_payment_method_limits(&self) -> Result<PaybisData<Vec<PaymentMethodWithLimits>>, reqwest::Error> {
        let url = format!("{PAYBIS_API_BASE_URL}/v2/public/payment-method-list-with-limits");
        self.client.get(url).query(&[("apikey", &self.api_key)]).send().await?.json().await
    }

    pub async fn create_request(&self, request_body: Request) -> Result<RequestResponse, Box<dyn std::error::Error + Send + Sync>> {
        let body_json = serde_json::to_string(&request_body)?;

        let signature = generate_rsa_pss_signature(&self.private_key, &body_json)?;

        let url = format!("{PAYBIS_API_BASE_URL}/v3/request");
        let response = self
            .client
            .post(url)
            .query(&[("partnerId", &self.api_key)])
            .header("X-Request-Signature", signature)
            .header("Content-Type", "application/json")
            .body(body_json)
            .send()
            .await?;

        let response_data: PaybisData<RequestResponse> = response.json().await?;
        Ok(response_data.data)
    }

    pub async fn get_buy_fiat_quote(
        &self,
        request: FiatBuyQuote,
        quote: PaybisQuote,
        user_ip: Option<String>,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let payment_method = quote.payment_methods.first().unwrap();
        let crypto_amount: f64 = payment_method.amount_to.amount.parse().unwrap_or(0.0);

        let crypto_value = BigNumberFormatter::f64_as_value(crypto_amount, request.asset.decimals as u32).unwrap_or_default();

        let redirect_url = self
            .get_redirect_url(
                &request.wallet_address,
                request.fiat_currency.as_ref(),
                &quote.currency_code_to,
                request.fiat_amount,
                true,
                user_ip,
            )
            .await?;

        Ok(FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency.as_ref().to_string(),
            crypto_amount,
            crypto_value,
            redirect_url,
        })
    }

    pub async fn get_sell_fiat_quote(
        &self,
        request: FiatSellQuote,
        quote: PaybisQuote,
        user_ip: Option<String>,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let payment_method = quote.payment_methods.first().unwrap();
        let fiat_amount: f64 = payment_method.amount_to.amount.parse().unwrap_or(0.0);

        let redirect_url = self
            .get_redirect_url(
                &request.wallet_address,
                &quote.currency_code_to,
                request.fiat_currency.as_ref(),
                request.crypto_amount,
                false,
                user_ip,
            )
            .await?;

        Ok(FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Sell,
            fiat_amount,
            fiat_currency: request.fiat_currency.as_ref().to_string(),
            crypto_amount: request.crypto_amount,
            crypto_value: request.crypto_value,
            redirect_url,
        })
    }

    pub async fn get_redirect_url(
        &self,
        wallet_address: &str,
        from_currency: &str,
        to_currency: &str,
        amount: f64,
        is_buy: bool,
        user_ip: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = if is_buy {
            Request::new_buy(wallet_address.to_string(), from_currency.to_string(), to_currency.to_string(), amount, user_ip)
        } else {
            Request::new_sell(wallet_address.to_string(), to_currency.to_string(), from_currency.to_string(), amount, user_ip)
        };

        let response = self.create_request(request_body).await?;

        let mut url = Url::parse(PAYBIS_WIDGET_URL)?;
        url.query_pairs_mut().append_pair("requestId", &response.request_id);

        Ok(url.to_string())
    }
}
