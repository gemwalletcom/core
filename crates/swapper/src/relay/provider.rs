use std::sync::Arc;

use alloy_primitives::U256;
use async_trait::async_trait;
use gem_client::Client;
use primitives::{AssetId, Chain, ChainType, SolanaInstruction, swap::ApprovalData};

use super::{
    Relay,
    asset::{SUPPORTED_CHAINS, asset_to_currency},
    chain::RelayChain,
    client::RelayClient,
    mapper,
    model::{RelayAppFee, RelayQuoteRequest, RelayQuoteResponse},
    tx_builder,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperQuoteData,
    approval::check_approval_erc20, config::get_swap_api_url, fees::resolve_max_quote_amount, referrer::DEFAULT_REFERRER,
};

fn resolve_app_fees(request: &QuoteRequest) -> Vec<RelayAppFee> {
    let Some(fees) = &request.options.fee else {
        return vec![];
    };
    let fee = &fees.evm;
    if fee.address.is_empty() || fee.bps == 0 {
        return vec![];
    }
    vec![RelayAppFee {
        recipient: fee.address.clone(),
        fee: fee.bps.to_string(),
    }]
}

impl Relay<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let url = get_swap_api_url("relay");
        let client = RelayClient::new(RpcClient::new(url, rpc_provider.clone()));
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

        let origin_currency = asset_to_currency(&from_asset_id, &from_chain)?;
        let destination_currency = asset_to_currency(&to_asset_id, &to_chain)?;
        let app_fees = resolve_app_fees(request);
        let amount = resolve_max_quote_amount(request)?;

        let relay_request = RelayQuoteRequest {
            user: request.wallet_address.clone(),
            origin_chain_id: from_chain.chain_id(),
            destination_chain_id: to_chain.chain_id(),
            origin_currency,
            destination_currency,
            amount: amount.clone(),
            recipient: request.destination_address.clone(),
            trade_type: "EXACT_INPUT".to_string(),
            referrer: if app_fees.is_empty() { None } else { Some(DEFAULT_REFERRER.to_string()) },
            app_fees,
            refund_to: Some(request.wallet_address.clone()),
            include_compute_unit_limit: from_chain == RelayChain::Solana,
            max_route_length: 6,
        };

        let quote_response = self.client.get_quote(relay_request).await?;

        let to_value = quote_response.details.currency_out.amount.clone();
        let eta_in_seconds = quote_response.details.time_estimate_u32();

        let quote = Quote {
            from_value: amount,
            to_value,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: from_asset_id,
                    output: to_asset_id,
                    route_data: serde_json::to_string(&quote_response).unwrap_or_default(),
                    gas_limit: None,
                }],
                slippage_bps: quote_response.details.slippage_bps().unwrap_or(request.options.slippage.bps),
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

        if from_chain == RelayChain::Solana {
            return self.build_solana_quote_data(quote, &quote_response).await;
        }

        let from_asset_id = quote.request.from_asset.asset_id();
        let approval = self.check_evm_approval(quote, &quote_response, &from_asset_id).await?;
        mapper::map_quote_data(&from_chain, &quote_response.steps, &quote.from_value, &quote_response.fees, approval)
    }

    async fn get_swap_result(&self, _chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let response = self.client.get_request(transaction_hash).await?;
        let request = response.requests.first().ok_or(SwapperError::InvalidRoute)?;
        Ok(mapper::map_swap_result(request))
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<Vec<String>, SwapperError> {
        Ok(super::RELAY_CONTRACTS.iter().map(|s| s.to_string()).collect())
    }
}

impl<C> Relay<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    async fn build_solana_quote_data(&self, quote: &Quote, quote_response: &RelayQuoteResponse) -> Result<SwapperQuoteData, SwapperError> {
        let step_data = mapper::get_step_data(&quote_response.steps)?;
        let instructions_json = step_data.instructions.as_ref().ok_or(SwapperError::InvalidRoute)?;
        let mut instructions: Vec<SolanaInstruction> = serde_json::from_value(instructions_json.clone()).map_err(|_| SwapperError::InvalidRoute)?;
        let lookup_addresses = step_data.address_lookup_table_addresses.as_deref().unwrap_or_default();

        let gas_limit = tx_builder::get_unit_limit(&instructions).map(|v| v.to_string());
        tx_builder::ensure_compute_unit_price(&mut instructions);

        let tx_data = tx_builder::build_solana_tx(&quote.request.wallet_address, &instructions, lookup_addresses, self.rpc_provider.clone()).await?;
        Ok(SwapperQuoteData::new_contract(String::new(), String::new(), tx_data, None, gas_limit))
    }

    async fn check_evm_approval(&self, quote: &Quote, quote_response: &RelayQuoteResponse, from_asset_id: &AssetId) -> Result<Option<ApprovalData>, SwapperError> {
        match from_asset_id.chain.chain_type() {
            ChainType::Ethereum if !from_asset_id.is_native() => {
                let router_address = quote_response
                    .steps
                    .iter()
                    .filter(|s| s.id != mapper::STEP_APPROVE)
                    .find_map(|s| s.items.as_ref()?.first().and_then(|item| item.data.as_ref().and_then(|d| d.to.clone())))
                    .ok_or(SwapperError::InvalidRoute)?;

                let token = from_asset_id.token_id.clone().ok_or(SwapperError::NotSupportedAsset)?;
                let amount: U256 = quote.from_value.parse().map_err(SwapperError::from)?;

                Ok(check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    token,
                    router_address,
                    amount,
                    self.rpc_provider.clone(),
                    &from_asset_id.chain,
                )
                .await?
                .approval_data())
            }
            _ => Ok(None),
        }
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
