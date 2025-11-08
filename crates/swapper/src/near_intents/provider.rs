use super::{
    AppFee, DepositMode, ExecutionStatus, NearIntentsClient, QuoteRequest as NearQuoteRequest, QuoteResponse, QuoteResponseError, QuoteResponseResult,
    SwapType, asset_id_from_near_intents, auto_quote_time_chains, deposit_memo_chains, get_near_intents_asset_id,
    model::{DEFAULT_REFERRAL, DEFAULT_WAIT_TIME_MS, DEPOSIT_TYPE_ORIGIN, RECIPIENT_TYPE_DESTINATION},
    reserved_tx_fees, supported_assets,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError,
    SwapperMode, SwapperProvider, SwapperQuoteAsset, SwapperQuoteData, client_factory::create_client_with_chain, near_intents::client::base_url,
};
use alloy_primitives::U256;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use gem_sui::{SuiClient, build_transfer_message_bytes};
use primitives::{Chain, swap::SwapStatus};
use std::{fmt::Debug, str::FromStr, sync::Arc};

const DEFAULT_DEADLINE_MINUTES: i64 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepositData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub memo: Option<String>,
}

pub struct NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: NearIntentsClient<C>,
    supported_assets: Vec<SwapperChainAsset>,
    sui_client: Arc<SuiClient<RpcClient>>,
}

impl<C> std::fmt::Debug for NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NearIntents")
            .field("provider", &self.provider)
            .field("client", &self.client)
            .field("supported_assets", &self.supported_assets)
            .field("sui_client", &"SuiClient::<RpcClient>")
            .finish()
    }
}

impl NearIntents<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let client = NearIntentsClient::new(RpcClient::new(base_url(), rpc_provider.clone()), None);
        let sui_client = Arc::new(SuiClient::new(create_client_with_chain(rpc_provider.clone(), Chain::Sui)));
        Self::with_client(client, sui_client)
    }

    pub fn boxed(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn crate::swapper_trait::Swapper> {
        Box::new(Self::new(rpc_provider))
    }
}

impl<C> NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn with_client(client: NearIntentsClient<C>, sui_client: Arc<SuiClient<RpcClient>>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::NearIntents),
            client,
            supported_assets: supported_assets(),
            sui_client,
        }
    }
    fn resolve_quote_amount(request: &QuoteRequest, mode: &SwapType) -> Result<String, SwapperError> {
        let base_amount = match mode {
            SwapType::ExactInput => request.value.clone(),
            SwapType::FlexInput => request.value.clone(),
        };

        if !request.options.use_max_amount || !request.from_asset.asset_id().is_native() {
            return Ok(base_amount);
        }

        let Some(reserved_base_amount) = reserved_tx_fees(request.from_asset.chain()) else {
            return Ok(base_amount);
        };

        let reserved_fee = Self::parse_u256(reserved_base_amount, "reservedFee")?;
        let amount_u256 = Self::parse_u256(&base_amount, "amount")?;

        if amount_u256 <= reserved_fee {
            return Err(SwapperError::InputAmountTooSmall);
        }

        Ok((amount_u256 - reserved_fee).to_string())
    }

    fn parse_u256(value: &str, field: &str) -> Result<U256, SwapperError> {
        U256::from_str(value).map_err(|_| SwapperError::ComputeQuoteError(format!("Invalid {field} value: {value}")))
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

    fn build_quote_request(&self, request: &QuoteRequest, mode: SwapType, amount: String, dry: bool) -> Result<NearQuoteRequest, SwapperError> {
        let origin_asset = get_near_intents_asset_id(&request.from_asset)?;
        let destination_asset = get_near_intents_asset_id(&request.to_asset)?;
        let deposit_mode = Self::resolve_deposit_mode(&request.from_asset);
        let from_chain = request.from_asset.asset_id().chain;
        let to_chain = request.to_asset.asset_id().chain;
        let quote_waiting_time_ms = Self::resolve_quote_waiting_time(from_chain, to_chain);

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
            quote_waiting_time_ms,
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
        if deposit_memo_chains().contains(&asset.asset_id().chain) {
            DepositMode::Memo
        } else {
            DepositMode::Simple
        }
    }

    fn resolve_quote_waiting_time(from_chain: Chain, to_chain: Chain) -> u32 {
        if auto_quote_time_chains().contains(&from_chain) || auto_quote_time_chains().contains(&to_chain) {
            0
        } else {
            DEFAULT_WAIT_TIME_MS
        }
    }

    async fn build_deposit_data(
        &self,
        deposit_memo: Option<String>,
        from_asset: &SwapperQuoteAsset,
        wallet_address: &str,
        deposit_address: &str,
        amount_in: &str,
    ) -> Result<DepositData, SwapperError> {
        if from_asset.asset_id().chain == Chain::Sui {
            return self.build_sui_deposit_data(from_asset, wallet_address, deposit_address, amount_in).await;
        }

        Ok(DepositData {
            to: deposit_address.to_string(),
            value: amount_in.to_string(),
            data: String::new(),
            memo: deposit_memo,
        })
    }

    async fn build_sui_deposit_data(
        &self,
        from_asset: &SwapperQuoteAsset,
        wallet_address: &str,
        deposit_address: &str,
        amount_in: &str,
    ) -> Result<DepositData, SwapperError> {
        let amount = amount_in
            .parse::<u64>()
            .map_err(|_| SwapperError::ComputeQuoteError("Invalid Sui amount provided for deposit".into()))?;

        let message_bytes = build_transfer_message_bytes(
            self.sui_client.as_ref(),
            wallet_address,
            deposit_address,
            amount,
            from_asset.asset_id().token_id.as_deref(),
        )
        .await
        .map_err(|err| SwapperError::NetworkError(format!("Failed to build Sui deposit data: {err}")))?;

        Ok(DepositData {
            to: deposit_address.to_string(),
            value: amount_in.to_string(),
            data: message_bytes,
            memo: None,
        })
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
            SwapperMode::ExactIn => SwapType::FlexInput,
            SwapperMode::ExactOut => return Err(SwapperError::NotImplemented),
        };

        let amount = Self::resolve_quote_amount(request, &mode)?;
        let quote_request = self.build_quote_request(request, mode, amount.clone(), true)?;
        let response = Self::extract_quote(self.client.fetch_quote(&quote_request).await?)?;
        let amount_out = Self::parse_amount(&response.quote.amount_out, "amountOut")?;

        let eta = response.quote.time_estimate;
        let route_data = serde_json::to_string(&quote_request)?;

        Ok(Quote {
            from_value: amount,
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
        let request_deposit_mode = quote_request.deposit_mode.clone();
        quote_request.dry = false;

        let response: QuoteResponse = Self::extract_quote(self.client.fetch_quote(&quote_request).await?)?;
        let QuoteResponse {
            quote_request: _,
            quote: near_quote,
        } = response;

        let deposit_address = near_quote
            .deposit_address
            .ok_or_else(|| SwapperError::ComputeQuoteError("Missing depositAddress in Near Intents response".into()))?;
        let amount_in = Self::parse_amount(&near_quote.amount_in, "amountIn")?;
        let deposit_mode = near_quote
            .deposit_mode
            .or(Some(request_deposit_mode))
            .ok_or_else(|| SwapperError::ComputeQuoteError("Near Intents response missing deposit mode".into()))?;
        let from_asset = &quote.request.from_asset;

        let memo_required = deposit_memo_chains().contains(&from_asset.asset_id().chain);
        let deposit_memo = near_quote.deposit_memo.filter(|memo| !memo.is_empty());

        if memo_required && deposit_mode != DepositMode::Memo {
            return Err(SwapperError::ComputeQuoteError("Near Intents Stellar deposits require a memo".into()));
        }
        if memo_required && deposit_memo.is_none() {
            return Err(SwapperError::ComputeQuoteError("Near Intents Stellar deposit missing memo".into()));
        }

        let data = self
            .build_deposit_data(deposit_memo, from_asset, &quote.request.wallet_address, &deposit_address, &amount_in)
            .await?;

        Ok(SwapperQuoteData::new_tranfer(data.to, data.value, data.memo))
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
    use crate::{SwapperError, SwapperMode, SwapperQuoteAsset, models::Options};
    use primitives::{AssetId, Chain};
    use serde_json::json;

    fn build_quote_request(amount: &str, use_max: bool, chain: Chain) -> QuoteRequest {
        let from_asset = SwapperQuoteAsset::from(AssetId::from_chain(chain));
        let to_asset = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Near));

        let options = Options {
            use_max_amount: use_max,
            ..Default::default()
        };

        QuoteRequest {
            from_asset,
            to_asset,
            wallet_address: "wallet".into(),
            destination_address: "dest".into(),
            value: amount.into(),
            mode: SwapperMode::ExactIn,
            options,
        }
    }

    #[test]
    fn resolve_quote_amount_with_use_max_reserves_fee() {
        let reserve = U256::from_str(reserved_tx_fees(Chain::Ethereum).unwrap()).unwrap();
        let amount = (reserve + U256::from(500u64)).to_string();

        let request = build_quote_request(&amount, true, Chain::Ethereum);
        let result = NearIntents::<RpcClient>::resolve_quote_amount(&request, &SwapType::FlexInput).expect("expected amount to resolve");

        assert_eq!(result, (U256::from_str(&amount).unwrap() - reserve).to_string());
    }

    #[test]
    fn resolve_quote_amount_without_use_max_keeps_amount() {
        let amount = "123456";
        let request = build_quote_request(amount, false, Chain::Ethereum);
        let result = NearIntents::<RpcClient>::resolve_quote_amount(&request, &SwapType::FlexInput).expect("expected amount to resolve");

        assert_eq!(result, amount);
    }

    #[test]
    fn resolve_quote_amount_rejects_when_under_reserved() {
        let reserve = U256::from_str(reserved_tx_fees(Chain::Ethereum).unwrap()).unwrap();
        let request = build_quote_request(&reserve.to_string(), true, Chain::Ethereum);

        let err = NearIntents::<RpcClient>::resolve_quote_amount(&request, &SwapType::FlexInput).expect_err("expected error");

        assert!(matches!(err, SwapperError::InputAmountTooSmall));
    }

    #[test]
    fn decode_quote_response_error_message() {
        let payload = json!({
            "message": "Amount is too low for bridge, try at least 8516130",
        });

        let decoded: QuoteResponseResult = serde_json::from_value(payload).expect("failed to decode error payload");

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
            use_max_amount: false,
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

        let options = Options::new_with_slippage(SwapperSlippage {
            bps: 100,
            mode: SwapperSlippageMode::Exact,
        });

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
