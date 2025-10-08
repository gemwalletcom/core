use super::{
    PROGRAM_ADDRESS,
    client::JupiterClient,
    model::{DynamicSlippage, QuoteDataRequest, QuoteRequest as JupiterRequest, QuoteResponse},
};
use crate::{
    FetchQuoteData, Options, ProviderData, ProviderType, Quote, QuoteRequest, Route, Swapper, SwapperChainAsset, SwapperError, SwapperMode, SwapperProvider,
    SwapperQuoteData, SwapperSlippageMode,
};
use alloy_primitives::U256;
use async_trait::async_trait;
use gem_client::Client;
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcResult};
use gem_solana::{
    SolanaRpc, TOKEN_PROGRAM, USDC_TOKEN_MINT, USDS_TOKEN_MINT, USDT_TOKEN_MINT, WSOL_TOKEN_ADDRESS, get_pubkey_by_str,
    models::{AccountData, ValueResult},
};
use primitives::{AssetId, Chain};
use std::collections::HashSet;

pub(crate) const JUPITER_API_URL: &str = "https://lite-api.jup.ag";

#[derive(Debug)]
pub struct Jupiter<C, R>
where
    C: Client + Clone + Send + Sync + 'static,
    R: Client + Clone + Send + Sync + 'static,
{
    pub provider: ProviderType,
    pub fee_mints: HashSet<&'static str>,
    http_client: JupiterClient<C>,
    rpc_client: JsonRpcClient<R>,
}

impl<C, R> Jupiter<C, R>
where
    C: Client + Clone + Send + Sync + 'static,
    R: Client + Clone + Send + Sync + 'static,
{
    pub fn with_clients(http_client: JupiterClient<C>, rpc_client: JsonRpcClient<R>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Jupiter),
            fee_mints: HashSet::from([USDC_TOKEN_MINT, USDT_TOKEN_MINT, USDS_TOKEN_MINT, WSOL_TOKEN_ADDRESS]),
            http_client,
            rpc_client,
        }
    }

    pub fn api_url() -> &'static str {
        JUPITER_API_URL
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
            let fee_account = super::token_account::get_token_account(&fee.solana.address, mint, token_program);
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
        // if fee_mint is in preset, no need to fetch token program
        let token_program = if self.fee_mints.contains(fee_mint.as_str()) {
            return Ok(self.get_fee_token_account(options, fee_mint.as_str(), TOKEN_PROGRAM).unwrap_or_default());
        } else {
            self.fetch_token_program(&fee_mint).await?
        };

        let mut fee_account = self.get_fee_token_account(options, &fee_mint, &token_program).unwrap_or_default();
        if fee_account.is_empty() {
            return Ok(fee_account);
        }

        // check fee token account exists, if not, set fee_account to empty string
        let rpc_call = SolanaRpc::GetAccountInfo(fee_account.clone());
        let rpc_result: JsonRpcResult<ValueResult<Option<AccountData>>> = self.rpc_client.call_with_cache(&rpc_call, None).await.map_err(SwapperError::from)?;
        if matches!(rpc_result, JsonRpcResult::Error(_)) || matches!(rpc_result, JsonRpcResult::Value(ref resp) if resp.result.value.is_none()) {
            fee_account = String::from("");
        }
        Ok(fee_account)
    }
}

#[async_trait]
impl<C, R> Swapper for Jupiter<C, R>
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
        let platform_fee_bps = swap_options.fee.unwrap_or_default().solana.bps;

        let auto_slippage = match swap_options.slippage.mode {
            SwapperSlippageMode::Auto => true,
            SwapperSlippageMode::Exact => false,
        };

        let quote_request = JupiterRequest {
            input_mint: input_mint.clone(),
            output_mint: output_mint.clone(),
            amount: request.value.clone(),
            platform_fee_bps,
            slippage_bps,
            auto_slippage,
            max_auto_slippage_bps: slippage_bps,
        };
        let swap_quote = self.http_client.get_swap_quote(quote_request).await?;
        let computed_auto_slippage = swap_quote.computed_auto_slippage.unwrap_or(swap_quote.slippage_bps);

        // Updated docs: https://dev.jup.ag/docs/api/swap-api/quote
        // The value includes platform fees and DEX fees, excluding slippage.
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
                slippage_bps: computed_auto_slippage,
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

        let quote_response: QuoteResponse = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let fee_account = self
            .fetch_fee_account(&quote.request.mode, &quote.request.options, &input_mint, &output_mint)
            .await?;

        let dynamic_slippage = match quote.request.options.slippage.mode {
            SwapperSlippageMode::Auto => Some(DynamicSlippage {
                max_bps: quote.request.options.slippage.bps,
            }),
            SwapperSlippageMode::Exact => None,
        };

        let request = QuoteDataRequest {
            user_public_key: quote.request.wallet_address.clone(),
            fee_account,
            quote_response,
            prioritization_fee_lamports: 500_000,
            dynamic_slippage,
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

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{SwapperMode, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use primitives::AssetId;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_jupiter_provider_fetch_quote() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Jupiter::new(rpc_provider);

        let options = Options {
            slippage: 100.into(),
            fee: None,
            preferred_providers: vec![],
        };

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Solana, Some(USDC_TOKEN_MINT.to_string()))),
            wallet_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            destination_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            value: "1000000000".to_string(),
            mode: SwapperMode::ExactIn,
            options,
        };

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
}
