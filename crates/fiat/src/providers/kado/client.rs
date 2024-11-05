use super::{
    mapper,
    model::{Asset, Blockchain, Blockchains, Quote, QuoteData, QuoteQuery, Response},
};
use crate::model::{filter_token_id, FiatMapping, FiatProviderAsset};
use primitives::fiat_quote::FiatQuoteType;
use primitives::{FiatBuyRequest, FiatProviderName, FiatQuote};
use reqwest::Client;
use url::Url;

const API_BASE_URL: &str = "https://api.kado.money";
const REDIRECT_URL: &str = "https://app.kado.money";

pub struct KadoClient {
    pub client: Client,
    pub api_key: String,
}

impl KadoClient {
    pub const NAME: FiatProviderName = FiatProviderName::Kado;

    pub fn new(client: Client, api_key: String) -> Self {
        KadoClient { client, api_key }
    }

    pub async fn get_quote_buy(
        &self,
        fiat_currency: String,
        symbol: String,
        fiat_amount: f64,
        network: String,
    ) -> Result<QuoteData, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/v2/ramp/quote", API_BASE_URL);
        let query = QuoteQuery {
            transaction_type: "buy".to_string(),
            blockchain: network,
            asset: symbol,
            amount: fiat_amount,
            currency: fiat_currency,
        };
        let quote = self
            .client
            .get(&url)
            .query(&query)
            .send()
            .await?
            .json::<Response<QuoteData>>()
            .await?;
        Ok(quote.data)
    }

    // pub async fn get_assets(&self) -> Result<Vec<Asset>, Box<dyn std::error::Error + Send + Sync>> {
    //     let url = format!("{}/v1/ramp/supported-assets", API_BASE_URL);
    //     let response = self
    //         .client
    //         .get(&url)
    //         .send()
    //         .await?
    //         .json::<Response<Assets>>()
    //         .await?;
    //     Ok(response.data.assets)
    // }

    pub async fn get_blockchains(
        &self,
    ) -> Result<Vec<Blockchain>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/v1/ramp/blockchains", API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Response<Blockchains>>()
            .await?;
        Ok(response.data.blockchains)
    }

    pub fn map_blockchain(blockchain: Blockchain) -> Vec<FiatProviderAsset> {
        blockchain
            .clone()
            .associated_assets
            .into_iter()
            .flat_map(|x| Self::map_asset(blockchain.clone(), x))
            .collect()
    }

    pub fn map_asset(blockchain: Blockchain, asset: Asset) -> Option<FiatProviderAsset> {
        let chain = mapper::map_asset_chain(blockchain.network.clone());
        let token_id = if asset.address.is_none() {
            None
        } else {
            Some(asset.address.clone().unwrap())
        };
        let token_id = filter_token_id(token_id);
        Some(FiatProviderAsset {
            id: asset.clone().symbol + "_" + blockchain.network.as_str(),
            chain,
            token_id,
            symbol: asset.clone().symbol,
            network: Some(blockchain.network),
            enabled: true,
        })
    }

    pub fn get_fiat_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
        quote: Quote,
    ) -> FiatQuote {
        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: request.fiat_amount,
            fiat_currency: request.clone().fiat_currency,
            crypto_amount: quote
                .clone()
                .receive_unit_count_after_fees
                .amount
                .unwrap_or_default(),
            redirect_url: self.redirect_url(
                request.fiat_currency.as_str(),
                request.fiat_amount,
                request_map.clone(),
                request.wallet_address.as_str(),
            ),
        }
    }

    pub fn redirect_url(
        &self,
        fiat_currency: &str,
        fiat_amount: f64,
        fiat_mapping: FiatMapping,
        address: &str,
    ) -> String {
        let mut components = Url::parse(REDIRECT_URL).unwrap();
        components
            .query_pairs_mut()
            .append_pair("apiKey", self.api_key.as_str())
            .append_pair("product", "BUY")
            .append_pair("onPayAmount", &fiat_amount.to_string())
            .append_pair("onPayCurrency", fiat_currency)
            .append_pair("onRevCurrency", &fiat_mapping.symbol)
            .append_pair("onToAddress", address)
            .append_pair(
                "network",
                &fiat_mapping.network.unwrap_or_default().to_uppercase(),
            )
            .append_pair("mode", "minimal");

        return components.as_str().to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redirect_url() {
        let client = KadoClient {
            client: Client::new(),
            api_key: "API_KEY".to_string(),
        };
        let fiat_mapping = FiatMapping {
            symbol: "ETH".to_string(),
            network: Some("ethereum".to_string()),
        };

        let expected_url = "https://app.kado.money/?apiKey=API_KEY&product=BUY&onPayAmount=100&onPayCurrency=USD&onRevCurrency=ETH&onToAddress=0x1234567890abcdef&network=ETHEREUM&mode=minimal";

        let actual_url = client.redirect_url("USD", 100.0, fiat_mapping, "0x1234567890abcdef");

        assert_eq!(actual_url, expected_url);
    }
}
