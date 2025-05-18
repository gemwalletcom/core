use super::{client::JupiterClient, model::*, PROGRAM_ADDRESS};
use crate::{
    network::jsonrpc::{JsonRpcClient, JsonRpcResult},
    swapper::{Swapper, *},
};

use alloy_primitives::U256;
use async_trait::async_trait;
use gem_solana::{
    get_pubkey_by_str,
    model::{AccountData, ValueResult},
    SolanaRpc, TOKEN_PROGRAM, USDC_TOKEN_MINT, USDS_TOKEN_MINT, USDT_TOKEN_MINT, WSOL_TOKEN_ADDRESS,
};
use primitives::{AssetId, Chain};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Jupiter {
    pub provider: SwapProviderType,
    pub fee_mints: HashSet<&'static str>,
}

impl Default for Jupiter {
    fn default() -> Self {
        Self {
            provider: SwapProviderType::new(GemSwapProvider::Jupiter),
            fee_mints: HashSet::from([USDC_TOKEN_MINT, USDT_TOKEN_MINT, USDS_TOKEN_MINT, WSOL_TOKEN_ADDRESS]),
        }
    }
}

impl Jupiter {
    pub fn get_endpoint(&self) -> String {
        "https://lite-api.jup.ag".into()
    }

    pub fn get_asset_address(&self, asset_id: &str) -> Result<String, SwapperError> {
        get_pubkey_by_str(asset_id)
            .map(|x| x.to_string())
            .ok_or(SwapperError::InvalidAddress(asset_id.to_string()))
    }

    pub fn get_fee_mint(&self, mode: &GemSwapMode, input: &str, output: &str) -> String {
        match mode {
            GemSwapMode::ExactIn => {
                if self.fee_mints.contains(output) {
                    return output.to_string();
                }
                input.to_string()
            }
            GemSwapMode::ExactOut => input.to_string(),
        }
    }

    pub fn get_fee_token_account(&self, options: &GemSwapOptions, mint: &str, token_program: &str) -> Option<String> {
        if let Some(fee) = &options.fee {
            let fee_account = super::token_account::get_token_account(&fee.solana.address, mint, token_program);
            return Some(fee_account);
        }
        None
    }

    pub async fn fetch_token_program(&self, mint: &str, provider: Arc<dyn AlienProvider>) -> Result<String, SwapperError> {
        let rpc_call = SolanaRpc::GetAccountInfo(mint.to_string());
        let client = JsonRpcClient::new_with_chain(provider.clone(), Chain::Solana);
        let rpc_result: JsonRpcResult<ValueResult<Option<AccountData>>> =
            client.call_with_cache(&rpc_call, Some(u64::MAX)).await.map_err(SwapperError::from)?;
        let value = rpc_result.take()?;

        value
            .value
            .map(|x| x.owner)
            .ok_or(SwapperError::NetworkError("fetch_token_program error".to_string()))
    }

    pub async fn fetch_fee_account(
        &self,
        mode: &GemSwapMode,
        options: &GemSwapOptions,
        input_mint: &str,
        output_mint: &str,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<String, SwapperError> {
        let fee_mint = self.get_fee_mint(mode, input_mint, output_mint);
        // if fee_mint is in preset, no need to fetch token program
        let token_program = if self.fee_mints.contains(fee_mint.as_str()) {
            return Ok(self.get_fee_token_account(options, fee_mint.as_str(), TOKEN_PROGRAM).unwrap());
        } else {
            self.fetch_token_program(&fee_mint, provider.clone()).await?
        };

        let mut fee_account = self.get_fee_token_account(options, &fee_mint, &token_program).unwrap_or_default();
        if fee_account.is_empty() {
            return Ok(fee_account);
        }

        // check fee token account exists, if not, set fee_account to empty string
        let rpc_call = SolanaRpc::GetAccountInfo(fee_account.clone());
        let client = JsonRpcClient::new_with_chain(provider.clone(), Chain::Solana);
        let rpc_result: JsonRpcResult<ValueResult<Option<AccountData>>> = client.call(&rpc_call).await.map_err(SwapperError::from)?;
        if matches!(rpc_result, JsonRpcResult::Error(_)) || matches!(rpc_result, JsonRpcResult::Value(ref resp) if resp.result.value.is_none()) {
            fee_account = String::from("");
        }
        Ok(fee_account)
    }
}

#[async_trait]
impl Swapper for Jupiter {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![SwapChainAsset::All(Chain::Solana)]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let input_mint = self.get_asset_address(&request.from_asset.id)?;
        let output_mint = self.get_asset_address(&request.to_asset.id)?;
        let swap_options = request.options.clone();
        let slippage_bps = swap_options.slippage.bps;
        let platform_fee_bps = swap_options.fee.unwrap_or_default().solana.bps;

        let auto_slippage = match swap_options.slippage.mode {
            GemSlippageMode::Auto => true,
            GemSlippageMode::Exact => false,
        };

        let quote_request = QuoteRequest {
            input_mint: input_mint.clone(),
            output_mint: output_mint.clone(),
            amount: request.value.clone(),
            platform_fee_bps,
            slippage_bps,
            auto_slippage,
            max_auto_slippage_bps: slippage_bps,
        };
        let client = JupiterClient::new(self.get_endpoint(), provider.clone());
        let swap_quote = client.get_swap_quote(quote_request).await?;
        let computed_auto_slippage = swap_quote.computed_auto_slippage.unwrap_or(swap_quote.slippage_bps);

        // Updated docs: https://dev.jup.ag/docs/api/swap-api/quote
        // The value includes platform fees and DEX fees, excluding slippage.
        let out_amount: U256 = swap_quote.out_amount.parse().map_err(SwapperError::from)?;

        let quote = SwapQuote {
            from_value: request.value.clone(),
            to_value: out_amount.to_string(),
            data: SwapProviderData {
                provider: self.provider().clone(),
                routes: vec![SwapRoute {
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

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        if quote.data.routes.is_empty() {
            return Err(SwapperError::InvalidRoute);
        }
        let route = &quote.data.routes[0];
        let input_mint = route.input.token_id.clone().unwrap();
        let output_mint = route.output.token_id.clone().unwrap();

        let quote_response: QuoteResponse = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let fee_account = self
            .fetch_fee_account(&quote.request.mode, &quote.request.options, &input_mint, &output_mint, provider.clone())
            .await?;

        let dynamic_slippage = match quote.request.options.slippage.mode {
            GemSlippageMode::Auto => Some(DynamicSlippage {
                max_bps: quote.request.options.slippage.bps,
            }),
            GemSlippageMode::Exact => None,
        };

        let request = QuoteDataRequest {
            user_public_key: quote.request.wallet_address.clone(),
            fee_account,
            quote_response,
            prioritization_fee_lamports: 500_000,
            dynamic_slippage,
        };

        let client = JupiterClient::new(self.get_endpoint(), provider);
        let quote_data = client.get_swap_quote_data(request).await?;

        if let Some(simulation_error) = quote_data.simulation_error {
            return Err(SwapperError::TransactionError(simulation_error.error));
        }

        let data = SwapQuoteData {
            to: PROGRAM_ADDRESS.to_string(),
            value: "".to_string(),
            data: quote_data.swap_transaction,
            approval: None,
            gas_limit: None,
        };
        Ok(data)
    }
}
