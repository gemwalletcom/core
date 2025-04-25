use alloy_primitives::U256;
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;

use super::{
    client::ProxyClient,
    mayan::{MayanClientStatus, MayanExplorer},
    symbiosis::model::SymbiosisTransactionData,
};
use crate::{
    config::swap_config::DEFAULT_SWAP_FEE_BPS,
    network::AlienProvider,
    swapper::{
        approval::{evm::check_approval_erc20, tron::check_approval_tron},
        models::{ApprovalData, ApprovalType, SwapChainAsset},
        FetchQuoteData, GemSwapProvider, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, Swapper, SwapperError,
    },
    tron::client::TronGridClient,
};
use primitives::{
    swap::{Quote, QuoteData, QuoteRequest},
    Chain, ChainType,
};

pub const PROVIDER_API_URL: &str = "https://api.gemwallet.com/swapper";
const DEFAULT_GAS_LIMIT: u64 = 500000;

#[derive(Debug)]
pub struct ProxyProvider {
    pub provider: SwapProviderType,
    pub url: String,
    pub assets: Vec<SwapChainAsset>,
}

impl ProxyProvider {
    pub async fn check_approval(
        &self,
        quote: &SwapQuote,
        quote_data: &QuoteData,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<(Option<ApprovalData>, Option<String>), SwapperError> {
        let request = &quote.request;
        let from_asset = &request.from_asset;
        let chain = from_asset.chain();
        let chain_type = chain.chain_type();

        if from_asset.is_native() {
            return Ok((None, quote_data.limit.clone()));
        }

        let token = match from_asset.asset_id().token_id {
            Some(id) => id,
            None => return Ok((None, None)), // Should not happen if not native
        };
        let wallet_address = request.wallet_address.clone();
        let spender = quote_data.to.clone();
        let amount = U256::from_str(&quote.from_value).map_err(SwapperError::from)?;

        let (approval, gas_limit) = match chain_type {
            ChainType::Ethereum => self.check_evm_approval(wallet_address, token, spender, amount, provider, &chain).await?,
            ChainType::Tron => {
                self.check_tron_approval(wallet_address, token, amount, quote_data.limit.clone(), quote, provider)
                    .await?
            }
            _ => (ApprovalType::None, None),
        };

        Ok((approval.approval_data(), gas_limit))
    }

    async fn check_evm_approval(
        &self,
        wallet_address: String,
        token: String,
        spender: String,
        amount: U256,
        provider: Arc<dyn AlienProvider>,
        chain: &Chain,
    ) -> Result<(ApprovalType, Option<String>), SwapperError> {
        let approval = check_approval_erc20(wallet_address, token, spender, amount, provider, chain).await?;
        let gas_limit = if matches!(approval, ApprovalType::Approve(_)) {
            Some(DEFAULT_GAS_LIMIT.to_string())
        } else {
            None
        };
        Ok((approval, gas_limit))
    }

    async fn check_tron_approval(
        &self,
        wallet_address: String,
        token: String,
        amount: U256,
        default_fee_limit: Option<String>,
        quote: &SwapQuote,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<(ApprovalType, Option<String>), SwapperError> {
        let route_data = quote.data.routes.first().map(|r| r.route_data.clone()).ok_or(SwapperError::InvalidRoute)?;
        let proxy_quote: Quote = serde_json::from_str(&route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let spender = proxy_quote.route_data["approveTo"]
            .as_str()
            .ok_or(SwapperError::TransactionError("Failed to check approval without spender".to_string()))?;

        let approval = check_approval_tron(&wallet_address, &token, spender, amount, provider.clone()).await?;
        let fee_limit = if matches!(approval, ApprovalType::Approve(_)) {
            default_fee_limit
        } else {
            let tx_data: SymbiosisTransactionData = serde_json::from_value(proxy_quote.route_data["tx"].clone()).map_err(|_| SwapperError::InvalidRoute)?;
            let client = TronGridClient::new(provider.clone());
            let call_value = tx_data.value.unwrap_or_default();
            let energy = client
                .estimate_tron_energy(
                    &wallet_address,
                    &tx_data.to,
                    &tx_data.function_selector,
                    &tx_data.data,
                    tx_data.fee_limit.unwrap_or_default(),
                    &call_value,
                )
                .await?;
            Some(energy.to_string())
        };
        Ok((approval, fee_limit))
    }
}

#[async_trait]
impl Swapper for ProxyProvider {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        self.assets.clone()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let client = ProxyClient::new(provider);
        let quote_request = QuoteRequest {
            from_address: request.wallet_address.clone(),
            to_address: request.destination_address.clone(),
            from_asset: request.from_asset.clone(),
            to_asset: request.to_asset.clone(),
            from_value: request.value.clone(),
            referral_bps: DEFAULT_SWAP_FEE_BPS,
            slippage_bps: request.options.slippage.bps,
        };

        let quote = client.get_quote(&self.url, quote_request.clone()).await?;

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: quote.output_value.clone(),
            data: SwapProviderData {
                provider: self.provider().clone(),
                routes: vec![SwapRoute {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&quote).unwrap(),
                    gas_limit: None,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(quote.eta_in_seconds),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let routes = quote.data.clone().routes;
        let route_data: Quote = serde_json::from_str(&routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let client = ProxyClient::new(provider.clone());

        let data = client.get_quote_data(&self.url, route_data).await?;
        let (approval, gas_limit) = self.check_approval(quote, &data, provider).await?;

        Ok(SwapQuoteData {
            to: data.to,
            value: data.value,
            data: data.data,
            approval,
            gas_limit,
        })
    }

    async fn get_transaction_status(&self, _chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        match self.provider.id {
            GemSwapProvider::Mayan => {
                let client = MayanExplorer::new(provider);
                let result = client.get_transaction_status(transaction_hash).await?;
                Ok(result.client_status == MayanClientStatus::Completed)
            }
            _ => Ok(true),
        }
    }
}
