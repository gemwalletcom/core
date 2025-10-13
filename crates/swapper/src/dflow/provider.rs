use super::{
    PROGRAM_ADDRESS,
    client::DFlowClient,
    model::{QuoteDataRequest, QuoteRequest as DFlowRequest, QuoteResponse},
};
use crate::{
    FetchQuoteData, Options, ProviderData, ProviderType, Quote, QuoteRequest, Route, Swapper, SwapperChainAsset, SwapperError, SwapperMode, SwapperProvider,
    SwapperQuoteData,
};
use alloy_primitives::U256;
use async_trait::async_trait;
use gem_client::Client;
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcResult};
use gem_solana::{
    SolanaRpc, TOKEN_PROGRAM, USDC_TOKEN_MINT, USDS_TOKEN_MINT, USDT_TOKEN_MINT, WSOL_TOKEN_ADDRESS, get_pubkey_by_str,
    models::{AccountData, ValueResult},
    token_account::get_token_account,
};
use primitives::{AssetId, Chain};
use std::collections::HashSet;

pub(crate) const DFLOW_API_URL: &str = "https://quote-api.dflow.net";

#[derive(Debug)]
pub struct DFlow<C, R>
where
    C: Client + Clone + Send + Sync + 'static,
    R: Client + Clone + Send + Sync + 'static,
{
    pub provider: ProviderType,
    pub fee_mints: HashSet<&'static str>,
    http_client: DFlowClient<C>,
    rpc_client: JsonRpcClient<R>,
}

impl<C, R> DFlow<C, R>
where
    C: Client + Clone + Send + Sync + 'static,
    R: Client + Clone + Send + Sync + 'static,
{
    pub fn with_clients(http_client: DFlowClient<C>, rpc_client: JsonRpcClient<R>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::DFlow),
            fee_mints: HashSet::from([USDC_TOKEN_MINT, USDT_TOKEN_MINT, USDS_TOKEN_MINT, WSOL_TOKEN_ADDRESS]),
            http_client,
            rpc_client,
        }
    }

    pub fn api_url() -> &'static str {
        DFLOW_API_URL
    }

    pub fn get_asset_address(&self, asset_id: &str) -> Result<String, SwapperError> {
        get_pubkey_by_str(asset_id)
            .map(|x| x.to_string())
            .ok_or(SwapperError::InvalidAddress(asset_id.to_string()))
    }

    fn get_fee_mint(&self, mode: &SwapperMode, input: &str, output: &str) -> String {
        match mode {
            SwapperMode::ExactIn => {
                if self.fee_mints.contains(output) {
                    return output.to_string();
                }
                input.to_string()
            }
            SwapperMode::ExactOut => input.to_string(),
        }
    }

    fn get_fee_token_account(&self, options: &Options, mint: &str, token_program: &str) -> Option<String> {
        if let Some(fee) = &options.fee {
            let fee_account = get_token_account(&fee.solana.address, mint, token_program);
            return Some(fee_account);
        }
        None
    }

    async fn fetch_token_program(&self, mint: &str) -> Result<String, SwapperError> {
        let rpc_call = SolanaRpc::GetAccountInfo(mint.to_string());
        let rpc_result: JsonRpcResult<ValueResult<Option<AccountData>>> =
            self.rpc_client.call_with_cache(&rpc_call, Some(u64::MAX)).await.map_err(SwapperError::from)?;
        let value = rpc_result.take()?;

        value
            .value
            .map(|x| x.owner)
            .ok_or(SwapperError::NetworkError("fetch_token_program error".to_string()))
    }

    async fn fetch_fee_account(&self, mode: &SwapperMode, options: &Options, input_mint: &str, output_mint: &str) -> Result<String, SwapperError> {
        let fee_mint = self.get_fee_mint(mode, input_mint, output_mint);
        let token_program = if self.fee_mints.contains(fee_mint.as_str()) {
            return Ok(self.get_fee_token_account(options, fee_mint.as_str(), TOKEN_PROGRAM).unwrap_or_default());
        } else {
            self.fetch_token_program(&fee_mint).await?
        };

        let mut fee_account = self.get_fee_token_account(options, &fee_mint, &token_program).unwrap_or_default();
        if fee_account.is_empty() {
            return Ok(fee_account);
        }

        let rpc_call = SolanaRpc::GetAccountInfo(fee_account.clone());
        let rpc_result: JsonRpcResult<ValueResult<Option<AccountData>>> = self.rpc_client.call_with_cache(&rpc_call, None).await.map_err(SwapperError::from)?;
        if matches!(rpc_result, JsonRpcResult::Error(_)) || matches!(rpc_result, JsonRpcResult::Value(ref resp) if resp.result.value.is_none()) {
            fee_account = String::from("");
        }
        Ok(fee_account)
    }
}

#[async_trait]
impl<C, R> Swapper for DFlow<C, R>
where
    C: Client + Clone + Send + Sync + 'static,
    R: Client + Clone + Send + Sync + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Solana)]
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let input_mint = self.get_asset_address(&request.from_asset.id)?;
        let output_mint = self.get_asset_address(&request.to_asset.id)?;
        let swap_options = request.options.clone();
        let slippage_bps = swap_options.slippage.bps;
        let platform_fee_bps = swap_options.fee.as_ref().map(|f| f.solana.bps);

        let quote_request = DFlowRequest {
            input_mint: input_mint.clone(),
            output_mint: output_mint.clone(),
            amount: request.value.clone(),
            slippage_bps: Some(slippage_bps),
            platform_fee_bps,
        };

        let swap_quote = self.http_client.get_swap_quote(quote_request).await?;

        let out_amount: U256 = swap_quote.out_amount.parse().map_err(SwapperError::from)?;

        let quote = Quote {
            from_value: request.value.clone(),
            to_value: out_amount.to_string(),
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: AssetId::from(Chain::Solana, Some(input_mint)),
                    output: AssetId::from(Chain::Solana, Some(output_mint)),
                    route_data: serde_json::to_string(&swap_quote).unwrap_or_default(),
                    gas_limit: None,
                }],
                slippage_bps: swap_quote.slippage_bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        };
        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        if quote.data.routes.is_empty() {
            return Err(SwapperError::InvalidRoute);
        }
        let route = &quote.data.routes[0];
        let input_mint = route.input.token_id.clone().unwrap();
        let output_mint = route.output.token_id.clone().unwrap();

        let mut quote_response: QuoteResponse = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;

        // Add mode to platformFee if it exists
        if let Some(ref mut platform_fee) = quote_response.platform_fee {
            if platform_fee.mode.is_none() {
                let mode = match quote.request.mode {
                    SwapperMode::ExactIn => "ExactIn",
                    SwapperMode::ExactOut => "ExactOut",
                };
                platform_fee.mode = Some(mode.to_string());
            }
        }

        let fee_account = self
            .fetch_fee_account(&quote.request.mode, &quote.request.options, &input_mint, &output_mint)
            .await?;

        let request = QuoteDataRequest {
            user_public_key: quote.request.wallet_address.clone(),
            fee_account,
            quote_response,
            compute_unit_price: None,
            prioritization_fee_lamports: Some(500_000),
        };

        let quote_data = self.http_client.get_swap_quote_data(&request).await?;

        if let Some(simulation_error) = quote_data.simulation_error {
            return Err(SwapperError::TransactionError(simulation_error.error));
        }

        let data = SwapperQuoteData {
            to: PROGRAM_ADDRESS.to_string(),
            value: "".to_string(),
            data: quote_data.swap_transaction,
            approval: None,
            gas_limit: None,
        };
        Ok(data)
    }
}

impl DFlow<crate::alien::RpcClient, crate::alien::RpcClient> {
    pub fn new(provider: std::sync::Arc<dyn crate::alien::RpcProvider>) -> Self {
        let http_client = super::client::DFlowClient::new(crate::alien::RpcClient::new(DFLOW_API_URL.into(), provider.clone()));
        let rpc_client = create_solana_rpc_client(provider, "DFlow");
        Self::with_clients(http_client, rpc_client)
    }
}

fn create_solana_rpc_client(provider: std::sync::Arc<dyn crate::alien::RpcProvider>, provider_name: &str) -> JsonRpcClient<crate::alien::RpcClient> {
    let solana_endpoint = provider
        .get_endpoint(Chain::Solana)
        .unwrap_or_else(|_| panic!("Failed to get Solana endpoint for {} provider", provider_name));
    JsonRpcClient::new(crate::alien::RpcClient::new(solana_endpoint, provider))
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{SwapperMode, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use primitives::AssetId;
    use std::sync::Arc;

    fn create_test_quote_request() -> QuoteRequest {
        QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Solana, Some(USDC_TOKEN_MINT.to_string()))),
            wallet_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            destination_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            value: "1000000000".to_string(),
            mode: SwapperMode::ExactIn,
            options: Options {
                slippage: 100.into(),
                fee: None,
                preferred_providers: vec![],
            },
        }
    }

    #[tokio::test]
    async fn test_dflow_provider_fetch_quote() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = DFlow::new(rpc_provider);
        let request = create_test_quote_request();

        let quote = provider.fetch_quote(&request).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);

        let route = &quote.data.routes[0];
        assert_eq!(
            route.input,
            AssetId::from(Chain::Solana, Some("So11111111111111111111111111111111111111112".to_string()))
        );
        assert_eq!(route.output, AssetId::from(Chain::Solana, Some(USDC_TOKEN_MINT.to_string())));
        assert!(!route.route_data.is_empty());

        let quote_response: QuoteResponse = serde_json::from_str(&route.route_data)?;
        assert_eq!(quote_response.input_mint, "So11111111111111111111111111111111111111112");
        assert_eq!(quote_response.output_mint, USDC_TOKEN_MINT);

        Ok(())
    }

    #[tokio::test]
    async fn test_dflow_provider_fetch_quote_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = DFlow::new(rpc_provider);
        let request = create_test_quote_request();

        let quote = provider.fetch_quote(&request).await?;
        let quote_data = provider.fetch_quote_data(&quote, FetchQuoteData::None).await?;

        assert_eq!(quote_data.to, PROGRAM_ADDRESS);
        assert_eq!(quote_data.value, "");
        assert!(!quote_data.data.is_empty());
        assert!(quote_data.approval.is_none());
        assert!(quote_data.gas_limit.is_none());

        Ok(())
    }
}
