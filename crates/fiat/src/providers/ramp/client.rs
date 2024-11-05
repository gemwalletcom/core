use crate::model::FiatProviderAsset;
use bigdecimal::ToPrimitive;
use primitives::fiat_quote::FiatQuoteType;
use primitives::{BigNumberFormatter, Chain, FiatBuyRequest, FiatProviderName, FiatQuote};
use reqwest::Client;
use url::Url;

use super::model::{Quote, QuoteAsset, QuoteAssets, QuoteRequest};

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

    pub async fn get_supported_assets(
        &self,
        currency: String,
        ip_address: String,
    ) -> Result<QuoteAssets, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/host-api/v3/assets?currencyCode={}&userIp={}&withDisabled=false&withHidden=false",
            RAMP_API_BASE_URL, currency, ip_address
        );
        let assets = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<QuoteAssets>()
            .await?;
        Ok(assets)
    }

    pub fn map_asset(asset: QuoteAsset) -> Option<FiatProviderAsset> {
        let chain = Self::map_asset_chain(asset.chain.clone());
        let token_id = asset.token_id();
        Some(FiatProviderAsset {
            id: asset.crypto_asset_symbol(),
            chain,
            token_id,
            symbol: asset.symbol,
            network: Some(asset.chain),
            enabled: asset.enabled.unwrap_or(false),
        })
    }

    pub fn map_asset_chain(chain: String) -> Option<Chain> {
        match chain.as_str() {
            "ETH" => Some(Chain::Ethereum),
            "SOLANA" => Some(Chain::Solana),
            "OPTIMISM" => Some(Chain::Optimism),
            "MATIC" => Some(Chain::Polygon),
            "XRP" => Some(Chain::Xrp),
            "TRON" => Some(Chain::Tron),
            "ARBITRUM" => Some(Chain::Arbitrum),
            "BASE" => Some(Chain::Base),
            "LTC" => Some(Chain::Litecoin),
            "AVAX" => Some(Chain::AvalancheC),
            "BSC" => Some(Chain::SmartChain),
            "COSMOS" => Some(Chain::Cosmos),
            "BTC" => Some(Chain::Bitcoin),
            "DOGE" => Some(Chain::Doge),
            "FANTOM" => Some(Chain::Fantom),
            "TON" => Some(Chain::Ton),
            "XDAI" => Some(Chain::Gnosis),
            "NEAR" => Some(Chain::Near),
            "ZKSYNCERA" => Some(Chain::ZkSync),
            "LINEA" => Some(Chain::Linea),
            "CELO" => Some(Chain::Celo),
            _ => None,
        }
    }

    pub async fn get_client_quote(
        &self,
        request: QuoteRequest,
    ) -> Result<Quote, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/host-api/v3/onramp/quote/all?hostApiKey={}",
            RAMP_API_BASE_URL, self.api_key
        );
        let quote = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<Quote>()
            .await?;
        Ok(quote)
    }

    pub fn get_fiat_quote(&self, request: FiatBuyRequest, quote: Quote) -> FiatQuote {
        let crypto_amount = BigNumberFormatter::big_decimal_value(
            quote.clone().card_payment.crypto_amount.as_str(),
            quote.asset.decimals,
        )
        .unwrap_or_default();

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.clone().fiat_amount,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount: crypto_amount.to_f64().unwrap_or_default(),
            redirect_url: self.redirect_url(request.clone(), quote.clone()),
        }
    }

    pub fn redirect_url(&self, request: FiatBuyRequest, quote: Quote) -> String {
        let mut components = Url::parse(RAMP_REDIRECT_URL).unwrap();
        components
            .query_pairs_mut()
            .append_pair("hostApiKey", &self.api_key)
            .append_pair("defaultAsset", &quote.asset.crypto_asset_symbol())
            .append_pair("swapAsset", &quote.asset.crypto_asset_symbol())
            .append_pair("fiatCurrency", &request.clone().fiat_currency.to_string())
            .append_pair("fiatValue", &request.clone().fiat_amount.to_string())
            .append_pair("userAddress", request.wallet_address.as_str())
            .append_pair(
                "webhookStatusUrl",
                "https://api.gemwallet.com/v1/fiat/webhooks/ramp",
            );

        components.as_str().to_string()
    }
}
