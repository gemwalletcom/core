use super::{
    AppFee, DepositMode, NearIntentsClient, NearIntentsExplorer, QuoteRequest as NearQuoteRequest, QuoteResponse, QuoteResponseError, QuoteResponseResult, SwapType,
    auto_quote_time_chains, deposit_memo_chains, get_asset_id_from_near_intents, get_near_intents_asset_id,
    model::{DEFAULT_WAIT_TIME_MS, DEPOSIT_TYPE_ORIGIN, ExplorerTransaction, RECIPIENT_TYPE_DESTINATION},
    supported_assets,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperMode,
    SwapperProvider, SwapperQuoteAsset, SwapperQuoteData, amount_to_value,
    client_factory::create_client_with_chain,
    cross_chain::VaultAddresses,
    fees::DEFAULT_REFERRER,
    fees::resolve_max_quote_value,
    near_intents::client::{base_url, explorer_url},
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use gem_sui::{SuiClient, build_transfer_message_bytes};
use primitives::{Chain, TransactionSwapMetadata, swap::SwapStatus};
use std::{fmt::Debug, sync::Arc};

const DEFAULT_DEADLINE_MINUTES: i64 = 30;

// Supported-chain subset of https://docs.near-intents.org/security-compliance/treasury-addresses
const TREASURY_ADDRESSES: [&str; 16] = [
    "0x2CfF890f0378a11913B6129B2E97417a2c302680",                         // EVM chains
    "0x233c5370CCfb3cD7409d9A3fb98ab94dE94Cb4Cd",                         // Monad, XLayer
    "1C6XJtNXiuXvk4oUAVMkKF57CRpaTrN5Ra",                                 // Bitcoin
    "1LxByjYMdnogW9Nc73srT4NCbS8oPVaXvZ",                                 // Bitcoin Cash
    "DRmCnxzL9U11EJzLmWkm2ikaZikPFbLuQD",                                 // Dogecoin
    "LQjEMkuiA2pCwFeUPwsu6ktzUubBVLsahX",                                 // Litecoin
    "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML",                                // Zcash
    "intents.near",                                                       // NEAR
    "HWjmoUNYckccg9Qrwi43JTzBcGcM1nbdAtATf9GXmz16",                       // Solana
    "UQAfoBd_f0pIvNpUPAkOguUrFWpGWV9TWBeZs_5TXE95_trZ",                   // TON
    "GDJ4JZXZELZD737NVFORH4PSSQDWFDZTKW3AIDKHYQG23ZXBPDGGQBJK",           // Stellar
    "0x00ea18889868519abd2f238966cab9875750bb2859ed3a34debec37781520138", // Sui
    "0xd1a1c1804e91ba85a569c7f018bb7502d2f13d4742d2611953c9c14681af6446", // Aptos
    "TX5XiRXdyz7sdFwF5mnhT1QoGCpbkncpke",                                 // TRON
    "r9R8jciZBYGq32DxxQrBPi5ysZm67iQitH",                                 // XRP
    "addr1v8wfpcg4qfhmnzprzysj6j9c53u5j56j8rvhyjp08s53s6g07rfjm",         // Cardano
];

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
    explorer: NearIntentsExplorer<C>,
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
            .field("explorer", &self.explorer)
            .field("supported_assets", &self.supported_assets)
            .field("sui_client", &"SuiClient::<RpcClient>")
            .finish()
    }
}

impl NearIntents<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let client = NearIntentsClient::new(RpcClient::new(base_url(), rpc_provider.clone()), None);
        let explorer = NearIntentsExplorer::new(RpcClient::new(explorer_url(), rpc_provider.clone()));
        let sui_client = Arc::new(SuiClient::new(create_client_with_chain(rpc_provider.clone(), Chain::Sui)));
        Self::with_client(client, explorer, sui_client)
    }

    pub fn boxed(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn crate::swapper_trait::Swapper> {
        Box::new(Self::new(rpc_provider))
    }
}

impl<C> NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn with_client(client: NearIntentsClient<C>, explorer: NearIntentsExplorer<C>, sui_client: Arc<SuiClient<RpcClient>>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::NearIntents),
            client,
            explorer,
            supported_assets: supported_assets(),
            sui_client,
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

    fn build_quote_request(&self, request: &QuoteRequest, mode: SwapType, amount: String, dry: bool) -> Result<NearQuoteRequest, SwapperError> {
        let origin_asset = get_near_intents_asset_id(&request.from_asset)?;
        let destination_asset = get_near_intents_asset_id(&request.to_asset)?;
        let deposit_mode = Self::resolve_deposit_mode(&request.from_asset);
        let from_chain = request.from_asset.asset_id().chain;
        let to_chain = request.to_asset.asset_id().chain;
        let quote_waiting_time_ms = Some(Self::resolve_quote_waiting_time(from_chain, to_chain));

        let deadline = (Utc::now() + Duration::minutes(DEFAULT_DEADLINE_MINUTES)).to_rfc3339();

        Ok(NearQuoteRequest {
            origin_asset,
            destination_asset,
            amount,
            referral: DEFAULT_REFERRER.to_string(),
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
            "REFUNDED" | "SWAP_REFUNDED" => SwapStatus::Failed,
            "SWAP_FAILED" | "FAILED" | "SWAP_LIQUIDITY_TIMEOUT" | "SWAP_RISK_FAILED" => SwapStatus::Failed,
            "KNOWN_DEPOSIT_TX" | "PENDING_DEPOSIT" | "INCOMPLETE_DEPOSIT" | "PROCESSING" => SwapStatus::Pending,
            _ => SwapStatus::Pending,
        }
    }

    fn build_swap_metadata(tx: &ExplorerTransaction) -> Option<TransactionSwapMetadata> {
        let from_asset = get_asset_id_from_near_intents(&tx.origin_asset)?;
        let to_asset = get_asset_id_from_near_intents(&tx.destination_asset)?;
        Some(TransactionSwapMetadata {
            from_asset,
            from_value: tx.amount_in.clone(),
            to_asset,
            to_value: tx.amount_out.clone(),
            provider: Some(SwapperProvider::NearIntents.as_ref().to_string()),
        })
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

    async fn build_sui_deposit_data(&self, from_asset: &SwapperQuoteAsset, wallet_address: &str, deposit_address: &str, amount_in: &str) -> Result<DepositData, SwapperError> {
        let amount = amount_in
            .parse::<u64>()
            .map_err(|_| SwapperError::ComputeQuoteError("Invalid Sui amount provided for deposit".into()))?;

        let message_bytes = build_transfer_message_bytes(self.sui_client.as_ref(), wallet_address, deposit_address, amount, from_asset.asset_id().token_id.as_deref())
            .await
            .map_err(|err| SwapperError::TransactionError(format!("Failed to build Sui deposit data: {err}")))?;

        Ok(DepositData {
            to: deposit_address.to_string(),
            value: amount_in.to_string(),
            data: message_bytes,
            memo: None,
        })
    }

    fn extract_quote(response: QuoteResponseResult, from_decimals: u32) -> Result<QuoteResponse, SwapperError> {
        match response {
            QuoteResponseResult::Ok(quote) => Ok(*quote),
            QuoteResponseResult::Err(error) => Err(map_quote_error(&error, from_decimals)),
        }
    }
}

fn map_quote_error(error: &QuoteResponseError, from_decimals: u32) -> SwapperError {
    let lower = error.message.to_ascii_lowercase();
    if lower.contains("too low") {
        SwapperError::InputAmountError {
            min_amount: parse_min_amount(&error.message, from_decimals),
        }
    } else {
        SwapperError::ComputeQuoteError(format!("Near Intents quote error: {}", error.message))
    }
}

fn parse_min_amount(message: &str, decimals: u32) -> Option<String> {
    let marker = "try at least ";
    let lower = message.to_ascii_lowercase();
    let start = lower.find(marker)? + marker.len();
    let tail = message.get(start..)?;
    let token = extract_numeric_token(tail)?;
    amount_to_value(&token, decimals)
}

fn extract_numeric_token(message: &str) -> Option<String> {
    let mut current = String::new();

    for ch in message.chars() {
        if ch.is_ascii_digit() || ch == '.' || ch == ',' || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            return Some(current);
        }
    }

    if current.is_empty() { None } else { Some(current) }
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

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let mode = match request.mode {
            SwapperMode::ExactIn => SwapType::FlexInput,
            SwapperMode::ExactOut => return Err(SwapperError::NotSupportedAsset),
        };

        let amount = resolve_max_quote_value(request)?;
        let quote_request = self.build_quote_request(request, mode, amount.clone(), true)?;
        let response = Self::extract_quote(self.client.fetch_quote(&quote_request).await?, request.from_asset.decimals)?;
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
                }],
            },
            request: request.clone(),
            eta_in_seconds: Some(eta),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let mut quote_request: NearQuoteRequest = serde_json::from_str(&route.route_data)?;
        let request_deposit_mode = quote_request.deposit_mode.clone();
        quote_request.dry = false;

        let response: QuoteResponse = Self::extract_quote(self.client.fetch_quote(&quote_request).await?, quote.request.from_asset.decimals)?;
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

        let DepositData { to, value, data: payload, memo } = data;

        Ok(SwapperQuoteData {
            data: payload,
            ..SwapperQuoteData::new_tranfer(to, value, memo)
        })
    }

    async fn get_swap_result(&self, _chain: Chain, hash: &str) -> Result<SwapResult, SwapperError> {
        let Some(tx) = self.explorer.search_transaction(hash).await? else {
            return Ok(SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
            });
        };

        let status = Self::map_transaction_status(&tx.status);
        let metadata = Self::build_swap_metadata(&tx);

        Ok(SwapResult { status, metadata })
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        Ok(VaultAddresses {
            deposit: vec![],
            send: TREASURY_ADDRESSES.iter().map(|s| s.to_string()).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SwapperError, SwapperMode, SwapperQuoteAsset, fees::reserved_tx_fees, models::Options};
    use alloy_primitives::U256;
    use primitives::asset_constants::SMARTCHAIN_USDT_ASSET_ID;
    use primitives::{AssetId, Chain};
    use serde_json::json;

    use std::str::FromStr;

    fn status(json: &str) -> SwapResult {
        let transactions: Vec<ExplorerTransaction> = serde_json::from_str(json).unwrap();
        let tx = &transactions[0];
        let status = NearIntents::<RpcClient>::map_transaction_status(&tx.status);
        let metadata = NearIntents::<RpcClient>::build_swap_metadata(tx);
        SwapResult { status, metadata }
    }

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
        let result = resolve_max_quote_value(&request).expect("expected amount to resolve");

        assert_eq!(result, (U256::from_str(&amount).unwrap() - reserve).to_string());
    }

    #[test]
    fn resolve_quote_amount_without_use_max_keeps_amount() {
        let amount = "123456";
        let request = build_quote_request(amount, false, Chain::Ethereum);
        let result = resolve_max_quote_value(&request).expect("expected amount to resolve");

        assert_eq!(result, amount);
    }

    #[test]
    fn resolve_quote_amount_rejects_when_under_reserved() {
        let reserve = U256::from_str(reserved_tx_fees(Chain::Ethereum).unwrap()).unwrap();
        let request = build_quote_request(&reserve.to_string(), true, Chain::Ethereum);

        let err = resolve_max_quote_value(&request).expect_err("expected error");

        assert!(matches!(err, SwapperError::InputAmountError { .. }));
    }

    #[test]
    fn swap_result_avax_to_smartchain() {
        let result = status(include_str!("testdata/tx_status_avax_to_smartchain.json"));

        assert_eq!(
            result,
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::AvalancheC),
                    from_value: "28000000000000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::SmartChain),
                    to_value: "399605209991817".to_string(),
                    provider: Some("near_intents".to_string()),
                }),
            }
        );
    }

    #[test]
    fn swap_result_smartchain_to_bitcoin() {
        let result = status(include_str!("testdata/tx_status_smartchain_to_bitcoin.json"));

        assert_eq!(
            result,
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: SMARTCHAIN_USDT_ASSET_ID.clone(),
                    from_value: "82000000000000000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Bitcoin),
                    to_value: "120496".to_string(),
                    provider: Some("near_intents".to_string()),
                }),
            }
        );
    }

    #[test]
    fn swap_result_bitcoin_to_ton_refunded() {
        let result = status(include_str!("testdata/tx_status_bitcoin_to_ton_refunded.json"));

        assert_eq!(
            result,
            SwapResult {
                status: SwapStatus::Failed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Bitcoin),
                    from_value: "20000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Ton),
                    to_value: "10031752296".to_string(),
                    provider: Some("near_intents".to_string()),
                }),
            }
        );
    }

    #[test]
    fn map_transaction_status_values() {
        let map = NearIntents::<RpcClient>::map_transaction_status;

        assert_eq!(map("SUCCESS"), SwapStatus::Completed);
        assert_eq!(map("SWAP_COMPLETED"), SwapStatus::Completed);
        assert_eq!(map("SWAP_COMPLETED_TX"), SwapStatus::Completed);

        assert_eq!(map("FAILED"), SwapStatus::Failed);
        assert_eq!(map("SWAP_FAILED"), SwapStatus::Failed);
        assert_eq!(map("REFUNDED"), SwapStatus::Failed);
        assert_eq!(map("SWAP_REFUNDED"), SwapStatus::Failed);
        assert_eq!(map("SWAP_LIQUIDITY_TIMEOUT"), SwapStatus::Failed);
        assert_eq!(map("SWAP_RISK_FAILED"), SwapStatus::Failed);

        assert_eq!(map("PENDING_DEPOSIT"), SwapStatus::Pending);
        assert_eq!(map("PROCESSING"), SwapStatus::Pending);
        assert_eq!(map("KNOWN_DEPOSIT_TX"), SwapStatus::Pending);
        assert_eq!(map("INCOMPLETE_DEPOSIT"), SwapStatus::Pending);
        assert_eq!(map("UNKNOWN_STATUS"), SwapStatus::Pending);
    }

    #[test]
    fn decode_quote_response_error_message() {
        let payload = json!({
            "message": "Amount is too low for bridge, try at least 8516130",
        });

        let decoded: QuoteResponseResult = serde_json::from_value(payload).expect("failed to decode error payload");

        let QuoteResponseResult::Err(err) = decoded else {
            panic!("expected error variant");
        };
        assert_eq!(err.message, "Amount is too low for bridge, try at least 8516130");
        assert_eq!(
            map_quote_error(&err, 6),
            SwapperError::InputAmountError {
                min_amount: Some("8516130".into())
            }
        );
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod swap_integration_tests {
    use super::*;
    use crate::{FetchQuoteData, SwapperMode, SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode, alien::reqwest_provider::NativeProvider, models::Options};
    use primitives::{
        AssetId, Chain,
        asset_constants::{ARBITRUM_USDC_ASSET_ID, BASE_USDC_ASSET_ID},
    };
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
            from_asset: SwapperQuoteAsset::from(ARBITRUM_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "500000".to_string(),
            mode: SwapperMode::ExactIn,
            options,
        };

        let quote = provider.get_quote(&request).await?;
        assert!(!quote.to_value.is_empty());

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
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
            value: "20000000".to_string(),
            mode: SwapperMode::ExactIn,
            options,
        };

        let quote = match provider.get_quote(&request).await {
            Ok(quote) => quote,
            Err(SwapperError::ComputeQuoteError(_)) => return Ok(()),
            Err(error) => return Err(error),
        };
        let quote_data = match provider.get_quote_data(&quote, FetchQuoteData::None).await {
            Ok(data) => data,
            Err(SwapperError::TransactionError(_)) => return Ok(()),
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

        println!("swap_result: {swap_result:?}");

        Ok(())
    }
}
