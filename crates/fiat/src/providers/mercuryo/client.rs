use crate::model::FiatMapping;
use number_formatter::BigNumberFormatter;
use primitives::{FiatBuyQuote, FiatQuoteType, FiatSellQuote};
use primitives::{FiatProviderName, FiatQuoteOld};
use reqwest::Client;
use std::collections::HashMap;

use super::models::{Asset, Currencies, CurrencyLimits, MercuryoTransactionResponse, Quote, QuoteQuery, QuoteSellQuery, Response};
use super::widget::MercuryoWidget;

const MERCURYO_API_BASE_URL: &str = "https://api.mercuryo.io";
pub struct MercuryoClient {
    pub client: Client,
    // widget
    pub widget_id: String,
    pub secret_key: String,
    pub partner_token: String,
}

impl MercuryoClient {
    pub const NAME: FiatProviderName = FiatProviderName::Mercuryo;

    pub fn new(client: Client, widget_id: String, secret_key: String, partner_token: String) -> Self {
        MercuryoClient {
            client,
            widget_id,
            secret_key,
            partner_token,
        }
    }

    pub async fn get_quote_buy(&self, fiat_currency: String, symbol: String, fiat_amount: f64, network: String) -> Result<Quote, reqwest::Error> {
        let query = QuoteQuery {
            from: fiat_currency.clone(),
            to: symbol.clone(),
            amount: fiat_amount,
            network: network.clone(),
            widget_id: self.widget_id.clone(),
        };
        let url = format!("{MERCURYO_API_BASE_URL}/v1.6/widget/buy/rate");
        Ok(self.client.get(url.as_str()).query(&query).send().await?.json::<Response<Quote>>().await?.data)
    }

    pub async fn get_quote_sell(&self, fiat_currency: String, symbol: String, fiat_amount: f64, network: String) -> Result<Quote, reqwest::Error> {
        let query = QuoteSellQuery {
            from: symbol.clone(),
            to: fiat_currency.clone(),
            quote_type: "sell".to_string(),
            amount: fiat_amount,
            network: network.clone(),
            widget_id: self.widget_id.clone(),
        };
        let url = format!("{MERCURYO_API_BASE_URL}/v1.6/public/convert");

        Ok(self.client.get(url.as_str()).query(&query).send().await?.json::<Response<Quote>>().await?.data)
    }

    pub async fn get_assets(&self) -> Result<Vec<Asset>, reqwest::Error> {
        let url = format!("{MERCURYO_API_BASE_URL}/v1.6/lib/currencies");
        let response = self.client.get(&url).send().await?.json::<Response<Currencies>>().await?;
        Ok(response.data.config.crypto_currencies)
    }

    pub async fn get_currencies(&self) -> Result<Currencies, reqwest::Error> {
        let url = format!("{MERCURYO_API_BASE_URL}/v1.6/lib/currencies");
        let response = self.client.get(&url).send().await?.json::<Response<Currencies>>().await?;
        Ok(response.data)
    }

    pub async fn get_countries(&self) -> Result<Response<Vec<String>>, reqwest::Error> {
        let query = [("type", "alpha2")];
        self.client
            .get(format!("{MERCURYO_API_BASE_URL}/v1.6/public/card-countries"))
            .query(&query)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_transaction(&self, transaction_id: &str) -> Result<Response<Vec<MercuryoTransactionResponse>>, reqwest::Error> {
        let query = [("merchant_transaction_id", transaction_id)];
        self.client
            .get(format!("{MERCURYO_API_BASE_URL}/v1.6/sdk-partner/transactions"))
            .header("Sdk-Partner-Token", &self.partner_token)
            .query(&query)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_currency_limits(&self, from: String, to: String) -> Result<Response<HashMap<String, CurrencyLimits>>, reqwest::Error> {
        let query = [("from", from), ("to", to), ("widget_id", self.widget_id.clone())];
        let url = format!("{MERCURYO_API_BASE_URL}/v1.6/public/currency-limits");
        self.client.get(&url).query(&query).send().await?.json().await
    }

    pub fn get_fiat_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping, quote: Quote) -> FiatQuoteOld {
        let crypto_value = BigNumberFormatter::f64_as_value(quote.clone().amount, request.asset.decimals as u32).unwrap_or_default();
        let widget = MercuryoWidget::new(
            self.widget_id.clone(),
            self.secret_key.clone(),
            request.wallet_address.clone(),
            request.ip_address.clone(),
            quote.clone(),
            FiatQuoteType::Buy,
            request_map.asset_symbol.network.unwrap_or_default(),
        );

        FiatQuoteOld {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency.as_ref().to_string(),
            crypto_amount: quote.amount,
            crypto_value,
            redirect_url: widget.to_url(),
        }
    }

    pub fn get_fiat_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping, quote: Quote) -> FiatQuoteOld {
        let widget = MercuryoWidget::new(
            self.widget_id.clone(),
            self.secret_key.clone(),
            request.wallet_address.clone(),
            request.ip_address.clone(),
            quote.clone(),
            FiatQuoteType::Sell,
            request_map.asset_symbol.network.unwrap_or_default(),
        );

        FiatQuoteOld {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Sell,
            fiat_amount: quote.fiat_amount,
            fiat_currency: request.fiat_currency.as_ref().to_string(),
            crypto_amount: quote.amount,
            crypto_value: request.crypto_value,
            redirect_url: widget.to_url(),
        }
    }
}
