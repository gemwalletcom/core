use super::{client::JupiterClient, model::*, JUPITER_API_URL, PROGRAM_ADDRESS};
use crate::swapper::{GemSwapProvider, *};

use async_trait::async_trait;
use gem_solana::get_asset_address;
use primitives::{AssetId, Chain};

#[derive(Debug, Default)]
pub struct Jupiter {}

impl Jupiter {
    pub fn get_asset_address(&self, asset_id: &AssetId) -> Result<String, SwapperError> {
        get_asset_address(asset_id)
            .map(|x| x.to_string())
            .ok_or_else(|| SwapperError::InvalidAddress { address: asset_id.to_string() })
    }

    pub fn get_fee_account(&self, options: &Option<GemSwapOptions>, mint: &str) -> String {
        let mut fee_account = String::from("");
        if let Some(options) = options {
            if let Some(fee) = &options.fee {
                fee_account = super::referral::get_referral_account(&fee.solana.address, mint);
            }
        }
        fee_account
    }
}

#[async_trait]
impl GemSwapProvider for Jupiter {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Jupiter
    }

    fn supported_chains(&self) -> Vec<Chain> {
        vec![Chain::Solana]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let input_mint = self.get_asset_address(&request.from_asset)?;
        let output_mint = self.get_asset_address(&request.to_asset)?;
        let swap_options = request.options.clone().unwrap_or_default();
        let slippage_bps = swap_options.slippage_bps;
        let platform_fee_bps = swap_options.fee.unwrap_or_default().solana.bps;

        let quote_request = QuoteRequest {
            input_mint: input_mint.clone(),
            output_mint: output_mint.clone(),
            amount: request.value.clone(),
            platform_fee_bps,
            slippage_bps,
            only_direct_routes: false,
        };
        let client = JupiterClient::new(JUPITER_API_URL.into(), provider.clone());
        let swap_quote = client.get_swap_quote(quote_request).await?;

        let quote = SwapQuote {
            from_value: request.value.clone(),
            to_value: swap_quote.out_amount.clone(),
            data: SwapProviderData {
                provider: self.provider(),
                routes: vec![SwapRoute {
                    route_type: serde_json::to_string(&swap_quote).unwrap_or_default(),
                    input: input_mint,
                    output: output_mint,
                    fee_tier: String::from("0"),
                    gas_estimate: None,
                }],
            },
            approval: ApprovalType::None,
            request: request.clone(),
        };
        Ok(quote)
    }
    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        if quote.data.routes.is_empty() {
            return Err(SwapperError::InvalidRoute);
        }
        let route = &quote.data.routes[0];
        let quote_response: QuoteResponse = serde_json::from_str(&route.route_type).map_err(|_| SwapperError::InvalidRoute)?;
        let fee_account = self.get_fee_account(&quote.request.options, &quote_response.output_mint);

        let request = QuoteDataRequest {
            user_public_key: quote.request.wallet_address.clone(),
            fee_account,
            quote_response,
            prioritization_fee_lamports: 500_000,
        };
        let client = JupiterClient::new(JUPITER_API_URL.into(), provider);
        let quote_data = client.get_swap_quote_data(request).await?;

        let data = SwapQuoteData {
            to: PROGRAM_ADDRESS.to_string(),
            value: "".to_string(),
            data: quote_data.swap_transaction,
        };
        Ok(data)
    }
    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        Ok(true)
    }
}
