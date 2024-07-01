use std::str::FromStr;

use super::model::{QuoteRequest, SwapResponse, SwapResult, SwapSpender, Tokenlist};
use gem_evm::address::EthereumAddress;
use primitives::{AssetId, Chain, ChainType, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};
use swap_provider::SwapError;

pub struct OneInchClient {
    api_url: String,
    api_key: String,
    fee: f64,
    fee_referral_address: String,
    version: String,
    client: reqwest::Client,
}

const NATIVE_ADDRESS: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
pub const PROVIDER_NAME: &str = "1inch";

impl OneInchClient {
    pub fn new(api_url: String, api_key: String, fee: f64, fee_referral_address: String) -> Self {
        let client = reqwest::Client::builder().build().unwrap();

        Self {
            client,
            api_url,
            api_key,
            fee,
            fee_referral_address,
            version: "v5.2".to_string(),
        }
    }

    pub fn chains(&self) -> Vec<Chain> {
        vec![
            Chain::Ethereum,
            Chain::Arbitrum,
            Chain::Optimism,
            Chain::Polygon,
            Chain::SmartChain,
            Chain::AvalancheC,
            Chain::Base,
            Chain::Fantom,
            Chain::Gnosis,
            Chain::ZkSync,
        ]
    }

    pub async fn get_tokenlist(&self, chain_id: &str) -> Result<Tokenlist, SwapError> {
        let url = format!("{}/token/v1.2/{chain_id}", self.api_url);
        Ok(self
            .client
            .get(&url)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<Tokenlist>()
            .await?)
    }

    pub fn get_asset_ids_for_tokenlist(&self, chain: Chain, tokenlist: Tokenlist) -> Vec<AssetId> {
        tokenlist
            .into_iter()
            .flat_map(|x| match EthereumAddress::from_str(&x.0) {
                Ok(token_id) => Some(AssetId {
                    chain,
                    token_id: Some(token_id.to_checksum()),
                }),
                Err(_) => None,
            })
            .collect::<Vec<AssetId>>()
    }

    pub async fn get_quote(&self, quote: SwapQuoteProtocolRequest) -> Result<SwapQuote, SwapError> {
        let network_id = quote.from_asset.chain.network_id();
        let src = if quote.from_asset.clone().is_native() {
            NATIVE_ADDRESS.to_string()
        } else {
            quote.from_asset.clone().token_id.unwrap()
        };
        let dst = if quote.to_asset.clone().is_native() {
            NATIVE_ADDRESS.to_string()
        } else {
            quote.to_asset.clone().token_id.unwrap()
        };
        let quote_request = QuoteRequest {
            src: src.clone(),
            dst,
            from: quote.wallet_address.clone(),
            amount: quote.amount.clone(),
            slippage: 1.0,
            disable_estimate: false,
            fee: self.fee,
            referrer: self.fee_referral_address.clone(),
        };

        let data: Option<SwapQuoteData>;
        let to_amount: String;
        if quote.include_data {
            let swap_quote = self.get_swap_quote_data(quote_request, network_id).await?;
            data = swap_quote.tx.map(|value| value.get_data());
            to_amount = swap_quote.to_amount;
        } else {
            let tuple = self
                .get_swap_quote_and_spender(quote_request, network_id)
                .await?;
            to_amount = tuple.0.to_amount;
            data = Some(SwapQuoteData {
                to: tuple.1,
                value: "".to_string(),
                data: "".to_string(),
            });
        };

        let quote = SwapQuote {
            chain_type: ChainType::Ethereum,
            from_amount: quote.amount.clone(),
            to_amount,
            fee_percent: self.fee as f32,
            provider: PROVIDER_NAME.into(),
            data,
        };
        Ok(quote)
    }

    pub async fn get_swap_quote_and_spender(
        &self,
        request: QuoteRequest,
        network_id: &str,
    ) -> Result<(SwapResult, String), SwapError> {
        let spender = self.get_swap_spender(network_id).await?;
        let quote = self.get_swap_quote(request, network_id).await?;

        Ok((quote, spender))
    }

    pub async fn get_swap_quote(
        &self,
        request: QuoteRequest,
        network_id: &str,
    ) -> Result<SwapResult, SwapError> {
        let url = format!(
            "{}/swap/{}/{}/quote",
            self.api_url, self.version, network_id
        );
        Ok(self
            .client
            .get(&url)
            .query(&request)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<SwapResult>()
            .await?)
    }

    pub async fn get_swap_spender(&self, network_id: &str) -> Result<String, SwapError> {
        let url = format!(
            "{}/swap/{}/{}/approve/spender",
            self.api_url, self.version, network_id
        );
        Ok(self
            .client
            .get(&url)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<SwapSpender>()
            .await?
            .address)
    }

    pub async fn get_swap_quote_data(
        &self,
        request: QuoteRequest,
        network_id: &str,
    ) -> Result<SwapResult, SwapError> {
        let url = format!("{}/swap/{}/{}/swap", self.api_url, self.version, network_id);
        let response = self
            .client
            .get(&url)
            .query(&request)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<SwapResponse>()
            .await?;
        let result = match response {
            SwapResponse::Success(result) => result,
            SwapResponse::Error(error) => {
                return Err(format!(
                    "{} ({}): {}",
                    error.error, error.status_code, error.description
                )
                .into())
            }
        };
        Ok(result)
    }
}
