use super::models::{Asset, Currencies, MercuryoTransactionResponse, Quote, QuoteQuery, QuoteSellQuery, Response};
use crate::model::{FiatMapping, FiatProviderAsset};
use hex;
use number_formatter::BigNumberFormatter;
use primitives::{FiatBuyQuote, FiatQuoteType, FiatSellQuote};
use primitives::{FiatProviderName, FiatQuote};
use reqwest::Client;
use sha2::{Digest, Sha512};
use url::Url;

const MERCURYO_API_BASE_URL: &str = "https://api.mercuryo.io";
const MERCURYO_REDIRECT_URL: &str = "https://exchange.mercuryo.io";
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

    pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
        let chain = super::mapper::map_asset_chain(asset.network.clone());
        let token_id = if asset.contract.is_empty() { None } else { Some(asset.contract.clone()) };
        Some(FiatProviderAsset {
            id: asset.clone().currency + "_" + asset.network.as_str(),
            chain,
            token_id,
            symbol: asset.clone().currency,
            network: Some(asset.network),
            enabled: true,
            unsupported_countries: None,
            buy_limits: vec![],
            sell_limits: vec![],
        })
    }

    pub fn get_fiat_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping, quote: Quote) -> FiatQuote {
        let crypto_value = BigNumberFormatter::f64_as_value(quote.clone().amount, request.asset.decimals as u32).unwrap_or_default();
        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency.as_ref().to_string(),
            crypto_amount: quote.clone().amount,
            crypto_value,
            redirect_url: self.redirect_url(
                quote.clone(),
                FiatQuoteType::Buy,
                &request_map.network.unwrap_or_default(),
                request.wallet_address.as_str(),
            ),
        }
    }

    pub fn get_fiat_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping, quote: Quote) -> FiatQuote {
        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Sell,
            fiat_amount: quote.fiat_amount,
            fiat_currency: request.fiat_currency.as_ref().to_string(),
            crypto_amount: quote.amount,
            crypto_value: request.crypto_value,
            redirect_url: self.redirect_url(
                quote.clone(),
                FiatQuoteType::Sell,
                &request_map.network.unwrap_or_default(),
                request.wallet_address.as_str(),
            ),
        }
    }

    pub fn redirect_url(&self, quote: Quote, quote_type: FiatQuoteType, network: &str, address: &str) -> String {
        let mut components = Url::parse(MERCURYO_REDIRECT_URL).unwrap();
        let signature_content = format!("{}{}", address, self.secret_key);
        let signature = hex::encode(Sha512::digest(signature_content));
        let id = uuid::Uuid::new_v4().to_string();

        components
            .query_pairs_mut()
            .append_pair("widget_id", self.widget_id.as_str())
            .append_pair("merchant_transaction_id", id.as_str())
            .append_pair("currency", &quote.currency)
            .append_pair("address", address)
            .append_pair("network", network)
            .append_pair("signature", &signature);

        match quote_type {
            FiatQuoteType::Buy => components
                .query_pairs_mut()
                .append_pair("type", "buy")
                .append_pair("fiat_amount", &quote.fiat_amount.to_string()),
            FiatQuoteType::Sell => components
                .query_pairs_mut()
                .append_pair("type", "sell")
                .append_pair("amount", &quote.amount.to_string()),
        };

        components.as_str().to_string()
    }
}
