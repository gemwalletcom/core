use crate::model::FiatProviderAsset;
use primitives::{Chain, FiatBuyRequest, FiatProviderName, FiatQuote};
use reqwest::Client;
use url::Url;

use super::model::{Asset, TransakQuote, TransakResponse};

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
        let url = format!(
            "{}/api/v2/currencies/price?ipAddress={}&fiatCurrency={}&cryptoCurrency={}&isBuyOrSell=buy&fiatAmount={}&network={}&partnerApiKey={}",
            TRANSAK_API_URL, ip_address, fiat_currency, symbol, fiat_amount, network, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        let transak_quote = response
            .json::<TransakResponse<TransakQuote>>()
            .await?
            .response;
        Ok(transak_quote)
    }

    pub fn get_fiat_quote(&self, request: FiatBuyRequest, quote: TransakQuote) -> FiatQuote {
        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
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

    pub async fn get_supported_assets(&self) -> Result<Vec<Asset>, reqwest::Error> {
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
