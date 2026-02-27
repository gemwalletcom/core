use std::sync::Arc;

use alloy_primitives::U256;
use async_trait::async_trait;
use gem_client::Client;
use primitives::{Chain, ChainType};

use super::{
    RELAY_API_URL, Relay,
    asset::{SUPPORTED_CHAINS, map_asset_to_relay_currency},
    chain::RelayChain,
    client::RelayClient,
    mapper,
    model::{RelayQuoteRequest, RelayQuoteResponse},
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperQuoteData,
    approval::check_approval_erc20, config::ReferralFee, referrer::DEFAULT_REFERRER,
};

fn resolve_referral_fee(request: &QuoteRequest, to_chain: RelayChain) -> Option<&ReferralFee> {
    let fees = request.options.fee.as_ref()?;
    let fee = match to_chain {
        RelayChain::Bitcoin => return None,
        RelayChain::Solana => &fees.solana,
        _ if to_chain.is_evm() => &fees.evm,
        _ => return None,
    };

    if fee.address.is_empty() || fee.bps == 0 {
        return None;
    }

    Some(fee)
}

fn resolve_referrer_data(request: &QuoteRequest, to_chain: RelayChain) -> (Option<String>, Option<String>) {
    let fee = resolve_referral_fee(request, to_chain);
    let referrer_address = fee.map(|fee| fee.address.clone());
    let referrer = referrer_address.as_ref().map(|_| DEFAULT_REFERRER.to_string());

    (referrer, referrer_address)
}

impl Relay<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let client = RelayClient::new(RpcClient::new(RELAY_API_URL.to_string(), rpc_provider.clone()));
        Self::with_client(client, rpc_provider)
    }
}

#[async_trait]
impl<C> Swapper for Relay<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        SUPPORTED_CHAINS.clone()
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_chain = RelayChain::from_chain(&request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let to_chain = RelayChain::from_chain(&request.to_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;

        let from_asset_id = request.from_asset.asset_id();
        let to_asset_id = request.to_asset.asset_id();

        let origin_currency = map_asset_to_relay_currency(&from_asset_id, &from_chain)?;
        let destination_currency = map_asset_to_relay_currency(&to_asset_id, &to_chain)?;
        let (referrer, referrer_address) = resolve_referrer_data(request, to_chain);

        let relay_request = RelayQuoteRequest {
            user: request.wallet_address.clone(),
            origin_chain_id: from_chain.chain_id(),
            destination_chain_id: to_chain.chain_id(),
            origin_currency,
            destination_currency,
            amount: request.value.clone(),
            recipient: request.destination_address.clone(),
            trade_type: "EXACT_INPUT".to_string(),
            referrer,
            referrer_address,
            refund_to: Some(request.wallet_address.clone()),
        };

        let quote_response = self.client.get_quote(relay_request).await?;

        let to_value = quote_response.details.currency_out.amount.clone();
        let eta_in_seconds = quote_response.details.time_estimate_u32();

        let quote = Quote {
            from_value: request.value.clone(),
            to_value,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: from_asset_id,
                    output: to_asset_id,
                    route_data: serde_json::to_string(&quote_response).unwrap_or_default(),
                    gas_limit: None,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds,
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let quote_response: RelayQuoteResponse = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let from_chain = RelayChain::from_chain(&quote.request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let from_asset_id = quote.request.from_asset.asset_id();

        let approval = match from_asset_id.chain.chain_type() {
            ChainType::Ethereum if !from_asset_id.is_native() => {
                let router_address = quote_response
                    .steps
                    .iter()
                    .find_map(|s| s.items.first().and_then(|item| item.data.as_ref().map(|d| d.to.clone())))
                    .ok_or(SwapperError::InvalidRoute)?;

                let token = from_asset_id.token_id.clone().ok_or(SwapperError::NotSupportedAsset)?;
                let amount: U256 = quote.from_value.parse().map_err(SwapperError::from)?;

                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    token,
                    router_address,
                    amount,
                    self.rpc_provider.clone(),
                    &from_asset_id.chain,
                )
                .await?
                .approval_data()
            }
            _ => None,
        };

        mapper::map_quote_data(&from_chain, &quote_response.steps, &quote.from_value, approval)
    }

    async fn get_swap_result(&self, _chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let response = self.client.get_request(transaction_hash).await?;
        let request = response.requests.first().ok_or(SwapperError::InvalidRoute)?;
        Ok(mapper::map_swap_result(request))
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{SwapperMode, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use primitives::AssetId;

    #[tokio::test]
    async fn test_relay_btc_to_eth() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::asset::ETHEREUM_USDC_TOKEN_ID;

        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Bitcoin)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID)),
            wallet_address: "bc1q4vxn43l44h30nkluqfxd9eckf45vr2awz38lwa".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "2000000".to_string(),
            mode: SwapperMode::ExactIn,
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.fetch_quote(&request).await?;
        let quote_data = relay.fetch_quote_data(&quote, FetchQuoteData::None).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(!quote.to_value.is_empty());
        assert!(!quote_data.data.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_eth_to_sol() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            value: "10000000000000000".to_string(),
            mode: SwapperMode::ExactIn,
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.fetch_quote(&request).await?;
        let quote_data = relay.fetch_quote_data(&quote, FetchQuoteData::None).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert!(!quote_data.data.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_sol_to_arb() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Arbitrum)),
            wallet_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "100000000".to_string(),
            mode: SwapperMode::ExactIn,
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.fetch_quote(&request).await?;
        let quote_data = relay.fetch_quote_data(&quote, FetchQuoteData::None).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert!(!quote_data.data.is_empty());

        Ok(())
    }
}
