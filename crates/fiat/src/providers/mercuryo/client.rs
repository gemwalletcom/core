use crate::model::{FiatMapping, FiatProviderAsset};
use hex;
use primitives::{Chain, FiatBuyRequest, FiatProviderName, FiatQuote};
use reqwest::Client;
use sha2::{Digest, Sha512};
use url::Url;

use super::model::{Asset, Currencies, MercyryoQuote, Response};

const MERCURYO_API_BASE_URL: &str = "https://api.mercuryo.io";
const MERCURYO_REDIRECT_URL: &str = "https://exchange.mercuryo.io";
pub struct MercuryoClient {
    pub client: Client,
    // widget
    pub widget_id: String,
    pub secret_key: String,
}

impl MercuryoClient {
    pub const NAME: FiatProviderName = FiatProviderName::Mercuryo;

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
        let chain = Self::map_asset_chain(asset.network.clone());
        let token_id = if asset.contract.is_empty() {
            None
        } else {
            Some(asset.contract.clone())
        };
        Some(FiatProviderAsset {
            id: asset.clone().currency + "_" + asset.network.as_str(),
            chain,
            token_id,
            symbol: asset.clone().currency,
            network: Some(asset.network),
            enabled: true,
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
            "COSMOS" => Some(Chain::Cosmos),
            "AVALANCHE" => Some(Chain::AvalancheC),
            "RIPPLE" => Some(Chain::Xrp),
            "LITECOIN" => Some(Chain::Litecoin),
            "FANTOM" => Some(Chain::Fantom),
            "DOGECOIN" => Some(Chain::Doge),
            "CELESTIA" => Some(Chain::Celestia),
            "NEWTON" => Some(Chain::Ton),
            "NEAR_PROTOCOL" => Some(Chain::Near),
            "LINEA" => Some(Chain::Linea),
            _ => None,
        }
    }

    pub fn get_fiat_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
        quote: MercyryoQuote,
    ) -> FiatQuote {
        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
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
        let id = uuid::Uuid::new_v4().to_string();

        components
            .query_pairs_mut()
            .append_pair("widget_id", self.widget_id.as_str())
            .append_pair("merchant_transaction_id", id.as_str())
            .append_pair("fiat_amount", &quote.fiat_amount.to_string())
            .append_pair("currency", &quote.currency)
            .append_pair("address", &address)
            .append_pair("network", &network)
            .append_pair("signature", &signature);

        return components.as_str().to_string();
    }
}
