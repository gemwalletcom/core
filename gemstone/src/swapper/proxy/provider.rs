use alloy_primitives::U256;
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;

use super::{
    client::ProxyClient,
    mayan::{map_mayan_chain_to_chain, MayanClientStatus, MayanExplorer},
    near::OneClickApi,
    symbiosis::model::SymbiosisTransactionData,
};
use crate::{
    config::swap_config::DEFAULT_SWAP_FEE_BPS,
    network::AlienProvider,
    swapper::{
        approval::{evm::check_approval_erc20, tron::check_approval_tron},
        models::{ApprovalType, SwapperChainAsset},
        remote_models::SwapperProviderMode,
        FetchQuoteData, Swapper, SwapperApprovalData, SwapperError, SwapperProvider, SwapperProviderData, SwapperProviderType, SwapperQuote, SwapperQuoteData,
        SwapperQuoteRequest, SwapperRoute, SwapperSwapResult,
    },
    tron::client::TronClient,
};
use primitives::{
    swap::{ProxyQuote, ProxyQuoteRequest, SwapQuoteData},
    AssetId, Chain, ChainType,
};

pub const PROVIDER_API_URL: &str = "https://api.gemwallet.com/swapper";
const DEFAULT_GAS_LIMIT: u64 = 500000;

#[derive(Debug)]
pub struct ProxyProvider {
    pub provider: SwapperProviderType,
    pub url: String,
    pub assets: Vec<SwapperChainAsset>,
}

impl ProxyProvider {
    pub async fn check_approval(
        &self,
        quote: &SwapperQuote,
        quote_data: &SwapQuoteData,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<(Option<SwapperApprovalData>, Option<String>), SwapperError> {
        let request = &quote.request;
        let from_asset = request.from_asset.asset_id();

        match from_asset.chain.chain_type() {
            ChainType::Ethereum => {
                if from_asset.is_native() {
                    Ok((None, None))
                } else {
                    let token = from_asset.token_id.unwrap();
                    self.check_evm_approval(
                        request.wallet_address.clone(),
                        token,
                        quote_data.to.clone(),
                        U256::from_str(&quote.from_value).map_err(SwapperError::from)?,
                        provider,
                        &from_asset.chain,
                    )
                    .await
                }
            }
            ChainType::Tron => {
                let amount = U256::from_str(&quote.from_value).map_err(SwapperError::from)?;
                self.check_tron_approval(
                    &from_asset,
                    request.wallet_address.clone(),
                    amount,
                    quote_data.gas_limit.clone(),
                    quote,
                    provider,
                )
                .await
            }
            _ => Ok((None, None)),
        }
    }

    async fn check_evm_approval(
        &self,
        wallet_address: String,
        token: String,
        spender: String,
        amount: U256,
        provider: Arc<dyn AlienProvider>,
        chain: &Chain,
    ) -> Result<(Option<SwapperApprovalData>, Option<String>), SwapperError> {
        let approval = check_approval_erc20(wallet_address, token, spender, amount, provider, chain).await?;
        let gas_limit = if matches!(approval, ApprovalType::Approve(_)) {
            Some(DEFAULT_GAS_LIMIT.to_string())
        } else {
            None
        };
        Ok((approval.approval_data(), gas_limit))
    }

    async fn check_tron_approval(
        &self,
        from_asset: &AssetId,
        wallet_address: String,
        amount: U256,
        default_fee_limit: Option<String>,
        quote: &SwapperQuote,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<(Option<SwapperApprovalData>, Option<String>), SwapperError> {
        let route_data = quote.data.routes.first().map(|r| r.route_data.clone()).ok_or(SwapperError::InvalidRoute)?;
        let proxy_quote: ProxyQuote = serde_json::from_str(&route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let spender = proxy_quote.route_data["approveTo"]
            .as_str()
            .ok_or(SwapperError::TransactionError("Failed to check approval without spender".to_string()))?;

        let approval = if from_asset.is_native() {
            ApprovalType::None
        } else {
            let token = from_asset.token_id.clone().unwrap();
            check_approval_tron(&wallet_address, &token, spender, amount, provider.clone()).await?
        };

        let fee_limit = if matches!(approval, ApprovalType::Approve(_)) {
            default_fee_limit
        } else {
            let tx_data: SymbiosisTransactionData = serde_json::from_value(proxy_quote.route_data["tx"].clone()).map_err(|_| SwapperError::InvalidRoute)?;
            let client = TronClient::new(provider.clone());
            let call_value = tx_data.value.unwrap_or_default();
            let energy = client
                .estimate_energy(
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
        Ok((approval.approval_data(), fee_limit))
    }
}

#[async_trait]
impl Swapper for ProxyProvider {
    fn provider(&self) -> &SwapperProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        self.assets.clone()
    }

    async fn fetch_quote(&self, request: &SwapperQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapperQuote, SwapperError> {
        let client = ProxyClient::new(provider);
        let quote_request = ProxyQuoteRequest {
            from_address: request.wallet_address.clone(),
            to_address: request.destination_address.clone(),
            from_asset: request.from_asset.clone(),
            to_asset: request.to_asset.clone(),
            from_value: request.value.clone(),
            referral_bps: DEFAULT_SWAP_FEE_BPS,
            slippage_bps: request.options.slippage.bps,
        };

        let quote = client.get_quote(&self.url, quote_request.clone()).await?;

        Ok(SwapperQuote {
            from_value: request.value.clone(),
            to_value: quote.output_value.clone(),
            data: SwapperProviderData {
                provider: self.provider().clone(),
                routes: vec![SwapperRoute {
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

    async fn fetch_quote_data(&self, quote: &SwapperQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let routes = quote.data.clone().routes;
        let route_data: ProxyQuote = serde_json::from_str(&routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let client = ProxyClient::new(provider.clone());

        let data = client.get_quote_data(&self.url, route_data).await?;
        let (approval, gas_limit) = self.check_approval(quote, &data, provider).await?;

        Ok(SwapperQuoteData {
            to: data.to,
            value: data.value,
            data: data.data,
            approval,
            gas_limit,
        })
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<SwapperSwapResult, SwapperError> {
        match self.provider.id {
            SwapperProvider::Mayan => {
                let client = MayanExplorer::new(provider);
                let result = client.get_transaction_status(transaction_hash).await?;

                let swap_status = result.client_status.swap_status();
                let dest_chain = map_mayan_chain_to_chain(&result.dest_chain);

                let (to_chain, to_tx_hash) = match result.client_status {
                    MayanClientStatus::Completed => (dest_chain, result.fulfill_tx_hash),
                    MayanClientStatus::Refunded | MayanClientStatus::InProgress => (dest_chain, None),
                };

                Ok(SwapperSwapResult {
                    status: swap_status,
                    from_chain: chain,
                    from_tx_hash: transaction_hash.to_string(),
                    to_chain,
                    to_tx_hash,
                })
            }
            SwapperProvider::NearIntents => {
                let client = OneClickApi::new(provider);
                let result = client.get_transaction_status(transaction_hash).await?;
                Ok(SwapperSwapResult {
                    status: result.swap_status(),
                    from_chain: chain,
                    from_tx_hash: transaction_hash.to_string(),
                    to_chain: result.to_chain(),
                    to_tx_hash: result.to_tx_hash(),
                })
            }
            // For OnChain providers, use the default implementation
            _ => {
                if self.provider.mode() == SwapperProviderMode::OnChain {
                    Ok(self.get_onchain_swap_status(chain, transaction_hash))
                } else {
                    Err(SwapperError::NotImplemented)
                }
            }
        }
    }
}
