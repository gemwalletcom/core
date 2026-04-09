use primitives::FiatProviderName;
use reqwest::Client;
use std::collections::HashMap;

use super::mapper::map_sell_quote;
use super::models::{Asset, Currencies, CurrencyLimits, MercuryoResponse, Quote, QuoteQuery, QuoteSellQuery, Response};

const MERCURYO_API_BASE_URL: &str = "https://api.mercuryo.io";
pub struct MercuryoClient {
    pub client: Client,
    // widget
    pub widget_id: String,
    pub secret_key: String,
}

impl MercuryoClient {
    pub const NAME: FiatProviderName = FiatProviderName::Mercuryo;

    pub fn new(client: Client, widget_id: String, secret_key: String) -> Self {
        MercuryoClient { client, widget_id, secret_key }
    }

    pub async fn get_quote_buy(&self, fiat_currency: String, symbol: String, fiat_amount: f64, network: String) -> Result<Quote, Box<dyn std::error::Error + Send + Sync>> {
        let query = QuoteQuery {
            from: fiat_currency.clone(),
            to: symbol.clone(),
            amount: fiat_amount,
            network: network.clone(),
            widget_id: self.widget_id.clone(),
        };
        let url = format!("{MERCURYO_API_BASE_URL}/v1.6/widget/buy/rate");
        self.client.get(url.as_str()).query(&query).send().await?.json::<MercuryoResponse<Quote>>().await?.into()
    }

    pub async fn get_quote_sell(&self, fiat_currency: String, symbol: String, fiat_amount: f64, network: String) -> Result<Quote, Box<dyn std::error::Error + Send + Sync>> {
        let buy_quote = self.get_quote_buy(fiat_currency.clone(), symbol.clone(), fiat_amount, network.clone()).await?;
        let sell_quote = self.get_sell_rate(symbol, fiat_currency, buy_quote.amount, network).await?;

        Ok(map_sell_quote(buy_quote, sell_quote, fiat_amount))
    }

    async fn get_sell_rate(&self, symbol: String, fiat_currency: String, crypto_amount: f64, network: String) -> Result<Quote, Box<dyn std::error::Error + Send + Sync>> {
        let query = QuoteSellQuery {
            from: symbol,
            to: fiat_currency,
            quote_type: "sell".to_string(),
            amount: crypto_amount,
            network,
            widget_id: self.widget_id.clone(),
        };
        let url = format!("{MERCURYO_API_BASE_URL}/v1.6/public/convert");
        self.client.get(url.as_str()).query(&query).send().await?.json::<MercuryoResponse<Quote>>().await?.into()
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

    pub async fn get_currency_limits(&self, from: String, to: String) -> Result<Response<HashMap<String, CurrencyLimits>>, reqwest::Error> {
        let query = [("from", from), ("to", to), ("widget_id", self.widget_id.clone())];
        let url = format!("{MERCURYO_API_BASE_URL}/v1.6/public/currency-limits");
        self.client.get(&url).query(&query).send().await?.json().await
    }
}
