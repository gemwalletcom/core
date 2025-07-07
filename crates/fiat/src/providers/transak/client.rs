use super::model::{Asset, Country, Response, TransakQuote};
use crate::model::{filter_token_id, FiatProviderAsset};
use base64::{engine::general_purpose::STANDARD_NO_PAD as BASE64, Engine as _};
use number_formatter::BigNumberFormatter;
use primitives::{FiatBuyQuote, FiatQuoteType};
use primitives::{FiatProviderName, FiatQuote};
use reqwest::Client;
use url::Url;

const TRANSAK_API_URL: &str = "https://api.transak.com";
const TRANSAK_REDIRECT_URL: &str = "https://global.transak.com";

#[derive(Debug, Clone)]
pub struct TransakClient {
    pub client: Client,
    pub api_key: String,
}

impl TransakClient {
    pub const NAME: FiatProviderName = FiatProviderName::Transak;

    pub fn new(client: Client, api_key: String) -> Self {
        TransakClient { client, api_key }
    }

    pub async fn get_buy_quote(
        &self,
        symbol: String,
        fiat_currency: String,
        fiat_amount: f64,
        network: String,
        ip_address: String,
    ) -> Result<TransakQuote, reqwest::Error> {
        self.get_quote("buy", symbol, fiat_currency, Some(fiat_amount), None, network, ip_address).await
    }

    pub async fn get_quote(
        &self,
        quote_type: &str,
        symbol: String,
        fiat_currency: String,
        fiat_amount: Option<f64>,
        crypto_amount: Option<&str>,
        network: String,
        country_code: String,
    ) -> Result<TransakQuote, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/api/v1/pricing/public/quotes");
        let mut query = vec![
            ("isBuyOrSell", quote_type.to_string()),
            ("quoteCountryCode", country_code.to_string()),
            ("fiatCurrency", fiat_currency.to_string()),
            ("cryptoCurrency", symbol.to_string()),
            ("network", network.to_string()),
            ("partnerApiKey", self.api_key.to_string()),
        ];
        if let Some(amount) = fiat_amount {
            query.push(("fiatAmount", amount.to_string()));
        }
        if let Some(amount) = crypto_amount {
            query.push(("cryptoAmount", amount.to_string()));
        }

        Ok(self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .json::<Response<TransakQuote>>()
            .await?
            .response)
    }

    pub fn get_fiat_quote(&self, request: FiatBuyQuote, quote: TransakQuote) -> FiatQuote {
        let crypto_value = BigNumberFormatter::f64_as_value(quote.crypto_amount, request.asset.decimals as u32).unwrap_or_default();
        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency,
            crypto_amount: quote.crypto_amount,
            crypto_value,
            redirect_url: self.redirect_url(quote, request.wallet_address),
        }
    }

    pub fn redirect_url(&self, quote: TransakQuote, address: String) -> String {
        let mut components = Url::parse(TRANSAK_REDIRECT_URL).unwrap();

        components
            .query_pairs_mut()
            .append_pair("apiKey", self.api_key.as_str())
            .append_pair("fiatAmount", &quote.fiat_amount.to_string())
            .append_pair("fiatCurrency", &quote.fiat_currency)
            .append_pair("cryptoCurrencyCode", &quote.crypto_currency)
            .append_pair("network", &quote.network.to_string())
            .append_pair("disableWalletAddressForm", "true")
            .append_pair("walletAddress", &address);

        components.as_str().to_string()
    }

    pub async fn get_supported_assets(&self) -> Result<Response<Vec<Asset>>, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/api/v2/currencies/crypto-currencies");
        self.client.get(&url).send().await?.json().await
    }

    pub async fn get_countries(&self) -> Result<Response<Vec<Country>>, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/api/v2/countries");
        self.client.get(&url).send().await?.json().await
    }

    pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
        let chain = super::mapper::map_asset_chain(asset.clone());
        let token_id = filter_token_id(chain, asset.clone().address);

        Some(FiatProviderAsset {
            id: asset.clone().unique_id,
            chain,
            token_id,
            symbol: asset.clone().symbol,
            network: Some(asset.clone().network.name),
            enabled: asset.is_allowed,
            unsupported_countries: Some(asset.unsupported_countries()),
        })
    }

    pub fn decode_jwt_content(&self, jwt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".to_string().into());
        }
        let payload = parts[1];
        let payload = BASE64.decode(payload)?;
        Ok(String::from_utf8(payload)?)
    }
}
