use super::{
    AppFee, DepositMode, ExecutionStatus, NearIntentsClient, QuoteRequest as NearQuoteRequest, QuoteResponse, QuoteResponseError, QuoteResponseResult,
    SwapType, asset_id_from_near_intents, get_near_intents_asset_id,
    model::{DEFAULT_REFERRAL, DEFAULT_WAIT_TIME_MS, DEPOSIT_TYPE_ORIGIN, RECIPIENT_TYPE_DESTINATION},
    supported_assets,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError,
    SwapperMode, SwapperProvider, SwapperQuoteAsset, SwapperQuoteData, near_intents::client::DEFAULT_NEAR_INTENTS_BASE_URL,
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

    pub fn boxed(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn crate::swapper_trait::Swapper> {
        Box::new(Self::new(rpc_provider))
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

    fn build_app_fee(options: &QuoteRequest) -> Option<Vec<AppFee>> {
        let fee = options.options.fee.as_ref()?;

        let referral = if !fee.near.address.is_empty() && fee.near.bps > 0 {
            Some((&fee.near.address, fee.near.bps))
        } else {
            None
        };

        referral.map(|(address, bps)| {
            vec![AppFee {
                recipient: address.clone(),
                fee: bps,
            }]
        })
    }

    fn build_quote_request(&self, request: &QuoteRequest, mode: SwapType, dry: bool) -> Result<NearQuoteRequest, SwapperError> {
        let origin_asset = get_near_intents_asset_id(&request.from_asset)?;
        let destination_asset = get_near_intents_asset_id(&request.to_asset)?;
        let amount = match mode {
            SwapType::ExactInput => request.value.clone(),
            SwapType::FlexInput => request.value.clone(),
        };
        let deposit_mode = Self::resolve_deposit_mode(&request.from_asset);

        let deadline = (Utc::now() + Duration::minutes(DEFAULT_DEADLINE_MINUTES)).to_rfc3339();

        Ok(NearQuoteRequest {
            origin_asset,
            destination_asset,
            amount,
            referral: DEFAULT_REFERRAL.to_string(),
            recipient: request.destination_address.clone(),
            swap_type: mode,
            slippage_tolerance: request.options.slippage.bps,
            app_fees: Self::build_app_fee(request),
            deposit_type: DEPOSIT_TYPE_ORIGIN.to_string(),
            refund_to: request.wallet_address.clone(),
            refund_type: DEPOSIT_TYPE_ORIGIN.to_string(),
            recipient_type: RECIPIENT_TYPE_DESTINATION.to_string(),
            deadline,
            quote_waiting_time_ms: DEFAULT_WAIT_TIME_MS,
            dry,
            deposit_mode,
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

    fn resolve_destination_chain(result: &ExecutionStatus) -> Option<Chain> {
        result
            .quote_response
            .as_ref()
            .and_then(|response| asset_id_from_near_intents(&response.quote_request.destination_asset))
            .map(|asset| asset.chain)
    }

    fn resolve_deposit_mode(asset: &SwapperQuoteAsset) -> DepositMode {
        match asset.asset_id().chain {
            Chain::Stellar => DepositMode::Memo,
            _ => DepositMode::Simple,
        }
    }

    fn extract_quote(response: QuoteResponseResult) -> Result<QuoteResponse, SwapperError> {
        match response {
            QuoteResponseResult::Ok(quote) => Ok(*quote),
            QuoteResponseResult::Err(error) => Err(map_quote_error(&error)),
        }
    }
}

fn map_quote_error(error: &QuoteResponseError) -> SwapperError {
    let lower = error.message.to_ascii_lowercase();
    if lower.contains("too low") {
        SwapperError::InputAmountTooSmall
    } else {
        SwapperError::NetworkError(format!("Near Intents quote error: {}", error.message))
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
        let response = Self::extract_quote(self.client.fetch_quote(&quote_request).await?)?;
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
        let mut quote_request: NearQuoteRequest = serde_json::from_str(&route.route_data)?;
        quote_request.dry = false;

        let response: QuoteResponse = Self::extract_quote(self.client.fetch_quote(&quote_request).await?)?;
        let QuoteResponse { quote_request: _, quote } = response;

        let deposit_address = quote
            .deposit_address
            .ok_or_else(|| SwapperError::ComputeQuoteError("Missing depositAddress in Near Intents response".into()))?;
        let amount_in = Self::parse_amount(&quote.amount_in, "amountIn")?;
        let deposit_mode = quote.deposit_mode.unwrap_or_default();
        let data = match deposit_mode {
            DepositMode::Memo => quote
                .deposit_memo
                .ok_or_else(|| SwapperError::ComputeQuoteError("Missing depositMemo for MEMO deposit mode".into()))?,
            DepositMode::Simple => String::new(),
        };

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SwapperError;
    use serde_json::json;

    #[test]
    fn decode_quote_response_error_message() {
        let payload = json!({
            "message": "Amount is too low for bridge, try at least 8516130",
        });

        let decoded: QuoteResponseResult =
            serde_json::from_value(payload).expect("failed to decode error payload");

        match decoded {
            QuoteResponseResult::Err(err) => {
                assert_eq!(err.message, "Amount is too low for bridge, try at least 8516130");
                assert!(matches!(map_quote_error(&err), SwapperError::InputAmountTooSmall));
            }
            QuoteResponseResult::Ok(_) => panic!("expected error variant"),
        }
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
    async fn test_near_intents_stellar_requires_memo() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::new().set_debug(true));
        let provider = NearIntents::new(rpc_provider);

        let options = Options {
            slippage: SwapperSlippage {
                bps: 100,
                mode: SwapperSlippageMode::Exact,
            },
            fee: None,
            preferred_providers: vec![],
        };

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Stellar)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Near)),
            wallet_address: "GBZXN7PIRZGNMHGA3RSSOEV56YXG54FSNTJDGQI3GHDVBKSXRZ5B6KJT".to_string(),
            destination_address: "test.near".to_string(),
            value: "1000000".to_string(),
            mode: SwapperMode::ExactIn,
            options,
        };

        let quote = match provider.fetch_quote(&request).await {
            Ok(quote) => quote,
            Err(SwapperError::NetworkError(_)) => return Ok(()),
            Err(error) => return Err(error),
        };
        let quote_data = match provider.fetch_quote_data(&quote, FetchQuoteData::None).await {
            Ok(data) => data,
            Err(SwapperError::NetworkError(_)) => return Ok(()),
            Err(error) => return Err(error),
        };

        assert!(!quote_data.data.is_empty(), "expected deposit memo for Stellar swaps via Near Intents");

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
