use super::{
    NearIntentsAppFee, NearIntentsClient, NearIntentsExecutionStatus, NearIntentsQuoteRequest, NearIntentsQuoteResponse, SwapType, asset_id_from_near_intents,
    get_near_intents_asset_id, model::{DEFAULT_REFERRAL, DEFAULT_WAIT_TIME_MS, DEPOSIT_TYPE_ORIGIN, RECIPIENT_TYPE_DESTINATION}, supported_assets,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError,
    SwapperMode, SwapperProvider, SwapperQuoteData, SwapperSlippage, near_intents::client::DEFAULT_NEAR_INTENTS_BASE_URL,
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use primitives::{Chain, swap::SwapStatus};
use std::{fmt::Debug, sync::Arc};

const DEFAULT_DEADLINE_MINUTES: i64 = 30;

#[derive(Debug)]
pub struct NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: NearIntentsClient<C>,
    supported_assets: Vec<SwapperChainAsset>,
}

impl NearIntents<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let client = NearIntentsClient::new(RpcClient::new(DEFAULT_NEAR_INTENTS_BASE_URL.to_string(), rpc_provider), None);
        Self::with_internal_client(client)
    }
}

impl<C> NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn with_internal_client(client: NearIntentsClient<C>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::NearIntents),
            client,
            supported_assets: supported_assets(),
        }
    }

    fn build_slippage(slippage: &SwapperSlippage) -> f64 {
        slippage.bps as f64 / 100.0
    }

    fn build_app_fee(options: &QuoteRequest) -> Option<Vec<NearIntentsAppFee>> {
        let fee = options.options.fee.as_ref()?;

        let referral = if !fee.near.address.is_empty() && fee.near.bps > 0 {
            Some((&fee.near.address, fee.near.bps))
        } else {
            None
        };

        referral.map(|(address, bps)| {
            vec![NearIntentsAppFee {
                recipient: address.clone(),
                fee: bps,
            }]
        })
    }

    fn build_quote_request(&self, request: &QuoteRequest, mode: SwapType, dry: bool) -> Result<NearIntentsQuoteRequest, SwapperError> {
        let origin_asset = get_near_intents_asset_id(&request.from_asset)?;
        let destination_asset = get_near_intents_asset_id(&request.to_asset)?;
        let amount = match mode {
            SwapType::ExactInput => request.value.clone(),
            SwapType::FlexInput => request.value.clone(),
        };

        let deadline = (Utc::now() + Duration::minutes(DEFAULT_DEADLINE_MINUTES)).to_rfc3339();

        Ok(NearIntentsQuoteRequest {
            origin_asset,
            destination_asset,
            amount,
            referral: DEFAULT_REFERRAL.to_string(),
            recipient: request.destination_address.clone(),
            swap_type: mode,
            slippage_tolerance: Self::build_slippage(&request.options.slippage),
            app_fees: Self::build_app_fee(request),
            deposit_type: DEPOSIT_TYPE_ORIGIN.to_string(),
            refund_to: request.wallet_address.clone(),
            refund_type: DEPOSIT_TYPE_ORIGIN.to_string(),
            recipient_type: RECIPIENT_TYPE_DESTINATION.to_string(),
            deadline,
            quote_waiting_time_ms: DEFAULT_WAIT_TIME_MS,
            dry,
        })
    }

    fn parse_amount(value: &str, field: &str) -> Result<String, SwapperError> {
        if value.is_empty() {
            Err(SwapperError::ComputeQuoteError(format!("Missing {field} in Near Intents response")))
        } else {
            Ok(value.to_string())
        }
    }

    fn map_transaction_status(status: &str) -> SwapStatus {
        match status {
            "SWAP_COMPLETED" | "SWAP_COMPLETED_TX" | "SUCCESS" => SwapStatus::Completed,
            "REFUNDED" | "SWAP_REFUNDED" => SwapStatus::Refunded,
            "SWAP_FAILED" | "FAILED" | "SWAP_LIQUIDITY_TIMEOUT" | "SWAP_RISK_FAILED" => SwapStatus::Failed,
            "KNOWN_DEPOSIT_TX" | "PENDING_DEPOSIT" | "INCOMPLETE_DEPOSIT" | "PROCESSING" => SwapStatus::Pending,
            _ => SwapStatus::Pending,
        }
    }

    fn resolve_destination_chain(result: &NearIntentsExecutionStatus) -> Option<Chain> {
        result
            .quote_response
            .as_ref()
            .and_then(|response| asset_id_from_near_intents(&response.quote_request.destination_asset))
            .map(|asset| asset.chain)
    }
}

#[async_trait]
impl<C> Swapper for NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        self.supported_assets.clone()
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let mode = match request.mode {
            SwapperMode::ExactIn => SwapType::ExactInput,
            SwapperMode::ExactOut => return Err(SwapperError::NotImplemented),
        };

        let quote_request = self.build_quote_request(request, mode, true)?;
        let response = self.client.fetch_quote(&quote_request).await?;
        let amount_out = Self::parse_amount(&response.quote.amount_out, "amountOut")?;

        let eta = response.quote.time_estimate;
        let route_data = serde_json::to_string(&quote_request)?;

        Ok(Quote {
            from_value: request.value.clone(),
            to_value: amount_out,
            data: ProviderData {
                provider: self.provider.clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data,
                    gas_limit: None,
                }],
            },
            request: request.clone(),
            eta_in_seconds: Some(eta),
        })
    }

    async fn fetch_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let mut quote_request: NearIntentsQuoteRequest = serde_json::from_str(&route.route_data)?;
        quote_request.dry = false;

        let response: NearIntentsQuoteResponse = self.client.fetch_quote(&quote_request).await?;
        let deposit_address = response
            .quote
            .deposit_address
            .ok_or_else(|| SwapperError::ComputeQuoteError("Missing depositAddress in Near Intents response".into()))?;
        let amount_in = Self::parse_amount(&response.quote.amount_in, "amountIn")?;
        let data = response.quote.deposit_memo.unwrap_or_default();

        Ok(SwapperQuoteData {
            to: deposit_address,
            value: amount_in,
            data,
            approval: None,
            gas_limit: None,
        })
    }

    async fn get_swap_result(&self, chain: Chain, deposit_address: &str) -> Result<SwapResult, SwapperError> {
        let status = self.client.get_transaction_status(deposit_address).await?;

        let swap_status = Self::map_transaction_status(status.status.as_str());
        let destination_chain = Self::resolve_destination_chain(&status);

        let swap_details = status.swap_details.unwrap_or_default();

        let to_tx_hash = match swap_status {
            SwapStatus::Refunded => swap_details.origin_chain_tx_hashes.first().map(|entry| entry.hash.clone()),
            _ => swap_details.destination_chain_tx_hashes.first().map(|entry| entry.hash.clone()),
        };

        let to_chain = match swap_status {
            SwapStatus::Refunded => Some(chain),
            _ => destination_chain,
        };

        Ok(SwapResult {
            status: swap_status,
            from_chain: chain,
            from_tx_hash: deposit_address.to_string(),
            to_chain,
            to_tx_hash,
        })
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod swap_integration_tests {
    use super::*;
    use crate::{
        FetchQuoteData, SwapperMode, SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode, alien::reqwest_provider::NativeProvider, models::Options,
    };
    use primitives::{AssetId, Chain, swap::SwapStatus};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_near_intents_quote() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::new().set_debug(true));
        let provider = NearIntents::new(rpc_provider);

        use crate::config::get_swap_config;

        let swap_config = get_swap_config();
        let options = Options {
            slippage: SwapperSlippage {
                bps: 100,
                mode: SwapperSlippageMode::Exact,
            },
            fee: Some(swap_config.referral_fee),
            preferred_providers: vec![],
        };

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::new("arbitrum_0xaf88d065e77c8cc2239327c5edb3a432268e5831").unwrap()),
            to_asset: SwapperQuoteAsset::from(AssetId::new("solana_epjfwdd5aufqssqem2qn1xzybapc8g4weggkzwytdt1v").unwrap()),
            wallet_address: "0x2527D02599Ba641c19FEa793cD0F167589a0f10D".to_string(),
            destination_address: "13QkxhNMrTPxoCkRdYdJ65tFuwXPhL5gLS2Z5Nr6gjRK".to_string(),
            value: "500000".to_string(),
            mode: SwapperMode::ExactIn,
            options,
        };

        let quote = provider.fetch_quote(&request).await?;
        assert!(!quote.to_value.is_empty());

        let quote_data = provider.fetch_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(!quote_data.to.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_near_intents_status() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::new().set_debug(true));
        let provider = NearIntents::new(rpc_provider);
        let deposit_address = "18gB9wZz1Q4CzniurLye1KdUUqjWjo3ePr";

        let swap_result = provider.get_swap_result(Chain::Bitcoin, deposit_address).await?;

        if swap_result.status == SwapStatus::Completed {
            assert_eq!(swap_result.to_chain, Some(Chain::Ethereum));
        }

        println!("swap_result: {swap_result:?}");

        Ok(())
    }
}
