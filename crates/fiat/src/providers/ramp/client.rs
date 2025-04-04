use crate::model::{filter_token_id, FiatMapping, FiatProviderAsset};
use number_formatter::BigNumberFormatter;
use primitives::{FiatBuyQuote, FiatProviderName, FiatQuote, FiatSellQuote};
use primitives::{FiatQuoteType, FiatQuoteTypeResult};
use reqwest::Client;
use url::Url;

use super::mapper::map_asset_chain;
use super::model::{Country, QuoteAsset, QuoteAssets, QuoteBuy, QuoteRequest, QuoteSell};

pub struct RampClient {
    client: Client,
    api_key: String,
}

const RAMP_API_BASE_URL: &str = "https://api.ramp.network";
const RAMP_REDIRECT_URL: &str = "https://app.ramp.network";

impl RampClient {
    pub const NAME: FiatProviderName = FiatProviderName::Ramp;

    pub fn new(client: Client, api_key: String) -> RampClient {
        RampClient { client, api_key }
    }

    pub fn get_crypto_asset_symbol(&self, request_map: FiatMapping) -> String {
        format!("{}_{}", request_map.network.unwrap_or_default(), request_map.symbol)
    }

    pub async fn get_supported_buy_assets(&self, currency: String, ip_address: String) -> Result<QuoteAssets, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/host-api/v3/assets?currencyCode={}&userIp={}&withDisabled=false&withHidden=false",
            RAMP_API_BASE_URL, currency, ip_address
        );
        let assets = self.client.get(&url).send().await?.json::<QuoteAssets>().await?;
        Ok(assets)
    }

    pub async fn get_supported_sell_assets(&self, currency: String, ip_address: String) -> Result<QuoteAssets, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/host-api/v3/offramp/assets?currencyCode={}&userIp={}&withDisabled=false&withHidden=false",
            RAMP_API_BASE_URL, currency, ip_address
        );
        let assets = self.client.get(&url).send().await?.json::<QuoteAssets>().await?;
        Ok(assets)
    }

    pub async fn get_countries(&self) -> Result<Vec<Country>, reqwest::Error> {
        self.client
            .get(format!("{}/api/host-api/countries", RAMP_API_BASE_URL))
            .send()
            .await?
            .json()
            .await
    }

    pub fn map_asset(asset: QuoteAsset) -> Option<FiatProviderAsset> {
        let chain = map_asset_chain(asset.chain.clone());
        let token_id = filter_token_id(chain, asset.token_id());
        Some(FiatProviderAsset {
            id: asset.crypto_asset_symbol(),
            chain,
            token_id,
            symbol: asset.symbol,
            network: Some(asset.chain),
            enabled: asset.enabled.unwrap_or(false),
            unsupported_countries: None,
        })
    }

    pub async fn get_client_buy_quote(&self, request: QuoteRequest) -> Result<QuoteBuy, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/host-api/v3/onramp/quote/all?hostApiKey={}", RAMP_API_BASE_URL, self.api_key);
        Ok(self.client.post(&url).json(&request).send().await?.json().await?)
    }

    pub async fn get_client_sell_quote(&self, request: QuoteRequest) -> Result<QuoteSell, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/host-api/v3/offramp/quote/all?hostApiKey={}", RAMP_API_BASE_URL, self.api_key);
        Ok(self.client.post(&url).json(&request).send().await?.json().await?)
    }

    pub fn get_fiat_buy_quote(&self, request: FiatBuyQuote, quote: QuoteBuy) -> FiatQuote {
        let crypto_amount = BigNumberFormatter::value_as_f64(quote.clone().card_payment.crypto_amount.as_str(), quote.asset.decimals).unwrap_or_default();

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.clone().fiat_amount,
            crypto_value: quote.clone().card_payment.crypto_amount,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount,
            redirect_url: self.redirect_url(
                FiatQuoteTypeResult::Buy(request),
                &quote.asset.crypto_asset_symbol(),
                quote.clone().card_payment.crypto_amount.as_str(),
            ),
        }
    }

    pub fn get_fiat_sell_quote(&self, request: FiatSellQuote, quote: QuoteSell) -> FiatQuote {
        let crypto_amount = BigNumberFormatter::value_as_f64(quote.clone().card_payment.crypto_amount.as_str(), quote.asset.decimals).unwrap_or_default();

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Sell,
            fiat_amount: quote.card_payment.fiat_value,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount,
            crypto_value: quote.clone().card_payment.crypto_amount.as_str().to_string(),
            redirect_url: self.redirect_url(
                FiatQuoteTypeResult::Sell(request),
                &quote.asset.crypto_asset_symbol(),
                quote.clone().card_payment.crypto_amount.as_str(),
            ),
        }
    }

    // docs: https://docs.ramp.network/configuration
    pub fn redirect_url(&self, quote_type: FiatQuoteTypeResult, symbol: &str, crypto_amount: &str) -> String {
        let mut components = Url::parse(RAMP_REDIRECT_URL).unwrap();
        components
            .query_pairs_mut()
            .append_pair("hostApiKey", &self.api_key)
            .append_pair("hostAppName", "Gem Wallet")
            .append_pair("hostLogoUrl", "https://gemwallet.com/images/presskit/gemwallet-icon-256x256.png")
            .append_pair("variant", "hosted-mobile")
            .append_pair("swapAsset", symbol)
            .append_pair("userAddress", quote_type.get_wallet_address().as_str())
            .append_pair("defaultAsset", symbol);

        match quote_type {
            FiatQuoteTypeResult::Buy(request) => components
                .query_pairs_mut()
                .append_pair("defaultFlow", "ONRAMP")
                .append_pair("fiatCurrency", &request.clone().fiat_currency.to_string())
                .append_pair("fiatValue", &request.clone().fiat_amount.to_string())
                .append_pair("webhookStatusUrl", "https://api.gemwallet.com/v1/fiat/webhooks/ramp"),
            FiatQuoteTypeResult::Sell(_) => components
                .query_pairs_mut()
                .append_pair("defaultFlow", "OFFRAMP")
                .append_pair("swapAmount", crypto_amount)
                .append_pair("offrampAsset", symbol)
                .append_pair("offrampWebhookV3Url", "https://api.gemwallet.com/v1/fiat/webhooks/ramp"),
        };
        components.as_str().to_string()
    }
}
