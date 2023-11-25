use std::error::Error;

use reqwest::Client;
use serde::Deserialize;
use url::Url;
use primitives::{fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest, fiat_provider::FiatProviderName};
use crate::model::{FiatMapping, FiatClient};
use async_trait::async_trait;
use sha2::{Sha512, Digest};
use hex;

const MERCURYO_API_BASE_URL: &str = "https://api.mercuryo.io";
const MERCURYO_REDIRECT_URL: &str = "https://exchange.mercuryo.io";
#[derive(Debug, Deserialize)]
pub struct MercyryoResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MercyryoQuote {
    pub amount: String,
    pub currency: String,
    pub fiat_amount: String,
}

pub struct MercuryoClient {
    client: Client,
    // widget
    widget_id: String,
    secret_key: String,
}

#[async_trait]
impl FiatClient for MercuryoClient {
    fn name(&self) -> FiatProviderName {
        FiatProviderName::Mercuryo
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn Error>> {
        let url = format!(
            "{}/v1.6/widget/buy/rate?from={}&to={}&amount={}&widget_id={}",
            MERCURYO_API_BASE_URL, request.fiat_currency, request_map.symbol, request.fiat_amount, self.widget_id
        );
        let response = self.client.get(&url).send().await?;
        let quote = response.json::<MercyryoResponse<MercyryoQuote>>().await?.data;

        Ok(self.get_fiat_quote(request, quote))
    }
}

impl MercuryoClient {
    pub fn new(client: Client, widget_id: String, secret_key: String) -> Self {
        MercuryoClient {
            client,
            widget_id,
            secret_key,
        }
    }

    fn get_fiat_quote(&self, request: FiatBuyRequest, quote: MercyryoQuote) -> FiatQuote {
        FiatQuote{
            provider: self.name().as_fiat_provider(),
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency,
            crypto_amount: quote.clone().amount.parse::<f64>().unwrap_or_default(),
            redirect_url: self.redirect_url(quote.clone(), request.wallet_address),
        }
    }

    pub fn redirect_url(&self, quote: MercyryoQuote, address: String) -> String {
        let mut components = Url::parse(MERCURYO_REDIRECT_URL).unwrap();
        let signature_content = format!("{}{}", address, self.secret_key);
        let signature = hex::encode(Sha512::digest(signature_content));

        components.query_pairs_mut()
            .append_pair("widget_id", self.widget_id.as_str())
            .append_pair("fiat_amount", &quote.fiat_amount.to_string())
            .append_pair("currency", &quote.currency)
            .append_pair("address", &address)
            .append_pair("signature", &signature);
        
        return components.as_str().to_string()
    }
}