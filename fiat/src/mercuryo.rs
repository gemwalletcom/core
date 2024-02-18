use crate::model::{FiatClient, FiatMapping, FiatProviderAsset};
use async_trait::async_trait;
use hex;
use primitives::{
    fiat_provider::FiatProviderName, fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest,
    Chain,
};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha512};
use url::Url;

const MERCURYO_API_BASE_URL: &str = "https://api.mercuryo.io";
const MERCURYO_REDIRECT_URL: &str = "https://exchange.mercuryo.io";
#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MercyryoQuote {
    pub amount: String,
    pub currency: String,
    pub fiat_amount: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Currencies {
    pub config: Config,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub crypto_currencies: Vec<Asset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub currency: String,
    pub network: String,
    pub contract: String,
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
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_quote_buy(
                request.fiat_currency.clone(),
                request_map.symbol.clone(),
                request.fiat_amount,
                request_map.network.clone().unwrap_or_default(),
            )
            .await?;

        Ok(self.get_fiat_quote(request, request_map.clone(), quote))
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

impl MercuryoClient {
    pub fn new(client: Client, widget_id: String, secret_key: String) -> Self {
        MercuryoClient {
            client,
            widget_id,
            secret_key,
        }
    }

    pub async fn get_quote_buy(
        &self,
        fiat_currency: String,
        symbol: String,
        fiat_amount: f64,
        network: String,
    ) -> Result<MercyryoQuote, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/v1.6/widget/buy/rate?from={}&to={}&amount={}&network={}&widget_id={}",
            MERCURYO_API_BASE_URL, fiat_currency, symbol, fiat_amount, network, self.widget_id
        );
        let quote = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Response<MercyryoQuote>>()
            .await?;
        Ok(quote.data)
    }

    pub async fn get_assets(&self) -> Result<Vec<Asset>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/v1.6/lib/currencies", MERCURYO_API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Response<Currencies>>()
            .await?;
        Ok(response.data.config.crypto_currencies)
    }

    pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
        let chain = Self::map_asset_chain(asset.network.clone())?;
        let token_id = if asset.contract.is_empty() {
            None
        } else {
            Some(asset.contract.clone())
        };
        Some(FiatProviderAsset {
            chain,
            token_id,
            symbol: asset.currency,
            network: Some(asset.network),
        })
    }

    pub fn map_asset_chain(chain: String) -> Option<Chain> {
        match chain.as_str() {
            "BITCOIN" => Some(Chain::Bitcoin),
            "ETHEREUM" => Some(Chain::Ethereum),
            "OPTIMISM" => Some(Chain::Optimism),
            "ARBITRUM" => Some(Chain::Arbitrum),
            "BASE" => Some(Chain::Base),
            "TRON" => Some(Chain::Tron),
            "BINANCESMARTCHAIN" => Some(Chain::SmartChain),
            "SOLANA" => Some(Chain::Solana),
            "POLYGON" => Some(Chain::Polygon),
            "COSMOS " => Some(Chain::Cosmos),
            "AVALANCHE" => Some(Chain::AvalancheC),
            "RIPPLE" => Some(Chain::Xrp),
            "LITECOIN" => Some(Chain::Litecoin),
            "FANTOM" => Some(Chain::Fantom),
            "DOGECOIN" => Some(Chain::Doge),
            "CELESTIAL" => Some(Chain::Celestia),
            _ => None,
        }
    }

    fn get_fiat_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
        quote: MercyryoQuote,
    ) -> FiatQuote {
        FiatQuote {
            provider: self.name().as_fiat_provider(),
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency,
            crypto_amount: quote.clone().amount.parse::<f64>().unwrap_or_default(),
            redirect_url: self.redirect_url(
                quote.clone(),
                request_map.network.unwrap_or_default(),
                request.wallet_address,
            ),
        }
    }

    pub fn redirect_url(&self, quote: MercyryoQuote, network: String, address: String) -> String {
        let mut components = Url::parse(MERCURYO_REDIRECT_URL).unwrap();
        let signature_content = format!("{}{}", address, self.secret_key);
        let signature = hex::encode(Sha512::digest(signature_content));

        components
            .query_pairs_mut()
            .append_pair("widget_id", self.widget_id.as_str())
            .append_pair("fiat_amount", &quote.fiat_amount.to_string())
            .append_pair("currency", &quote.currency)
            .append_pair("address", &address)
            .append_pair("network", &network)
            .append_pair("signature", &signature);

        return components.as_str().to_string();
    }
}
