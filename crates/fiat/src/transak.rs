use crate::model::{FiatClient, FiatMapping, FiatProviderAsset};
use async_trait::async_trait;
use primitives::{
    fiat_provider::FiatProviderName, fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest,
    Chain,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

const TRANSAK_API_URL: &str = "https://api.transak.com";
const TRANSAK_REDIRECT_URL: &str = "https://global.transak.com";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransakQuote {
    pub quote_id: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_currency: String,
    pub crypto_amount: f64,
    pub network: String,
}

#[derive(Debug, Serialize)]
struct TransakPayload<'a> {
    ip_address: &'a str,
    fiat_currency: &'a str,
    crypto_currency: &'a str,
    is_buy_or_sell: &'a str,
    fiat_amount: f64,
    network: &'a str,
    partner_api_key: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct TransakResponse<T> {
    pub response: T,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub coin_id: String,
    pub symbol: String,
    pub network: AssetNetwork,
    pub address: Option<String>,
    pub is_allowed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetNetwork {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct TransakClient {
    client: Client,
    api_key: String,
}

#[async_trait]
impl FiatClient for TransakClient {
    fn name(&self) -> FiatProviderName {
        FiatProviderName::Transak
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/v2/currencies/price?ipAddress={}&fiatCurrency={}&cryptoCurrency={}&isBuyOrSell=buy&fiatAmount={}&network={}&partnerApiKey={}",
            TRANSAK_API_URL, request.ip_address, request.fiat_currency, request_map.symbol, request.fiat_amount, request_map.network.unwrap_or_default(), self.api_key
        );

        let response = self.client.get(&url).send().await?;
        let transak_quote = response
            .json::<TransakResponse<TransakQuote>>()
            .await?
            .response;

        Ok(self.get_fiat_quote(request, transak_quote))
    }

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_assets()
            .await?
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }
}

impl TransakClient {
    pub fn new(client: Client, api_key: String) -> Self {
        TransakClient { client, api_key }
    }

    fn get_fiat_quote(&self, request: FiatBuyRequest, quote: TransakQuote) -> FiatQuote {
        FiatQuote {
            provider: self.name().as_fiat_provider(),
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency,
            crypto_amount: quote.crypto_amount,
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

        return components.as_str().to_string();
    }

    async fn get_assets(&self) -> Result<Vec<Asset>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v2/currencies/crypto-currencies", TRANSAK_API_URL);
        let response = self.client.get(&url).send().await?;
        let assets = response
            .json::<TransakResponse<Vec<Asset>>>()
            .await?
            .response;
        Ok(assets)
    }

    pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
        let chain = Self::map_asset_chain(asset.clone())?;
        let token_id = asset.clone().address.filter(|contract_address| {
            !["0x0000000000000000000000000000000000000000"].contains(&contract_address.as_str())
        });

        Some(FiatProviderAsset {
            chain,
            token_id,
            symbol: asset.symbol,
            network: Some(asset.network.name),
            enabled: asset.is_allowed,
        })
    }

    pub fn map_asset_chain(asset: Asset) -> Option<Chain> {
        match asset.network.name.as_str() {
            "ethereum" => Some(Chain::Ethereum),
            "polygon" => Some(Chain::Polygon),
            "aptos" => Some(Chain::Aptos),
            "sui" => Some(Chain::Sui),
            "arbitrum" => Some(Chain::Arbitrum),
            "optimism" => Some(Chain::Optimism),
            "base" => Some(Chain::Base),
            "bsc" => Some(Chain::SmartChain),
            "tron" => Some(Chain::Tron),
            "solana" => Some(Chain::Solana),
            "avaxcchain" => Some(Chain::AvalancheC),
            "ton" => Some(Chain::Ton),
            "osmosis" => Some(Chain::Osmosis),
            "fantom" => Some(Chain::Fantom),
            "mainnet" => match asset.coin_id.as_str() {
                "bitcoin" => Some(Chain::Bitcoin),
                "litecoin" => Some(Chain::Litecoin),
                "ripple" => Some(Chain::Xrp),
                "dogecoin" => Some(Chain::Doge),
                "tron" => Some(Chain::Tron),
                "cosmos" => Some(Chain::Cosmos),
                _ => None,
            },
            _ => None,
        }
    }
}
