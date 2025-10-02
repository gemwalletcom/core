use alloy_primitives::U256;
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;

use super::{
    client::ProxyClient,
    mayan::{MayanClientStatus, MayanExplorer, map_mayan_chain_to_chain},
    symbiosis::model::SymbiosisTransactionData,
};
use crate::{
    config::swap_config::DEFAULT_SWAP_FEE_BPS,
    models::GemApprovalData,
    network::AlienProvider,
    swapper::{
        FetchQuoteData, Swapper, SwapperError, SwapperProvider, SwapperProviderData, SwapperProviderType, SwapperQuote, SwapperQuoteData, SwapperQuoteRequest,
        SwapperRoute, SwapperSwapResult,
        approval::{evm::check_approval_erc20, tron::check_approval_tron},
        models::{ApprovalType, SwapperChainAsset},
        remote_models::SwapperProviderMode,
    },
    tron::client::TronClient,
};
use primitives::{
    AssetId, Chain, ChainType,
    swap::{ProxyQuote, ProxyQuoteRequest, SwapQuoteData},
};

pub const PROVIDER_API_URL: &str = "https://api.gemwallet.com/swapper";
const DEFAULT_GAS_LIMIT: u64 = 500000;

#[derive(Debug)]
pub struct ProxyProvider {
    pub provider: SwapperProviderType,
    pub url: String,
    pub assets: Vec<SwapperChainAsset>,
    pub(crate) rpc_provider: Arc<dyn AlienProvider>,
}

impl ProxyProvider {
    pub async fn check_approval(&self, quote: &SwapperQuote, quote_data: &SwapQuoteData) -> Result<(Option<GemApprovalData>, Option<String>), SwapperError> {
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
                        &from_asset.chain,
                    )
                    .await
                }
            }
            ChainType::Tron => {
                let amount = U256::from_str(&quote.from_value).map_err(SwapperError::from)?;
                self.check_tron_approval(&from_asset, request.wallet_address.clone(), amount, quote_data.gas_limit.clone(), quote)
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
        chain: &Chain,
    ) -> Result<(Option<GemApprovalData>, Option<String>), SwapperError> {
        let approval = check_approval_erc20(wallet_address, token, spender, amount, self.rpc_provider.clone(), chain).await?;
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
    ) -> Result<(Option<GemApprovalData>, Option<String>), SwapperError> {
        let route_data = quote.data.routes.first().map(|r| r.route_data.clone()).ok_or(SwapperError::InvalidRoute)?;
        let proxy_quote: ProxyQuote = serde_json::from_str(&route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let spender = proxy_quote.route_data["approveTo"]
            .as_str()
            .ok_or(SwapperError::TransactionError("Failed to check approval without spender".to_string()))?;

        let approval = if from_asset.is_native() {
            ApprovalType::None
        } else {
            let token = from_asset.token_id.clone().unwrap();
            check_approval_tron(&wallet_address, &token, spender, amount, self.rpc_provider.clone()).await?
        };

        let fee_limit = if matches!(approval, ApprovalType::Approve(_)) {
            default_fee_limit
        } else {
            let tx_data: SymbiosisTransactionData = serde_json::from_value(proxy_quote.route_data["tx"].clone()).map_err(|_| SwapperError::InvalidRoute)?;
            let client = TronClient::new(self.rpc_provider.clone());
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

    async fn fetch_quote(&self, request: &SwapperQuoteRequest) -> Result<SwapperQuote, SwapperError> {
        let client = ProxyClient::new(self.rpc_provider.clone());
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

    async fn fetch_quote_data(&self, quote: &SwapperQuote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let routes = quote.data.clone().routes;
        let route_data: ProxyQuote = serde_json::from_str(&routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let client = ProxyClient::new(self.rpc_provider.clone());

        let data = client.get_quote_data(&self.url, route_data).await?;
        let (approval, gas_limit) = self.check_approval(quote, &data).await?;

        Ok(SwapperQuoteData {
            to: data.to,
            value: data.value,
            data: data.data,
            approval,
            gas_limit,
        })
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapperSwapResult, SwapperError> {
        match self.provider.id {
            SwapperProvider::Mayan => {
                let client = MayanExplorer::new(self.rpc_provider.clone());
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

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{
        network::alien_provider::NativeProvider,
        swapper::{SwapperMode, asset::SUI_USDC_TOKEN_ID, models::SwapperOptions, remote_models::SwapperQuoteAsset},
    };
    use primitives::AssetId;

    #[tokio::test]
    async fn test_mayan_provider_fetch_quote() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = ProxyProvider::new_mayan(rpc_provider);

        let options = SwapperOptions {
            slippage: 200.into(),
            fee: None,
            preferred_providers: vec![],
        };

        let request = SwapperQuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            value: "50000000000000000".to_string(),
            mode: SwapperMode::ExactIn,
            options,
        };

        let quote = provider.fetch_quote(&request).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);
        assert_eq!(quote.data.slippage_bps, 200);
        assert!(quote.eta_in_seconds.is_some());

        let route = &quote.data.routes[0];
        assert_eq!(route.input, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(route.output, AssetId::from_chain(Chain::Solana));
        assert!(!route.route_data.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_cetus_provider_fetch_quote() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = ProxyProvider::new_cetus_aggregator(rpc_provider);

        let options = SwapperOptions {
            slippage: 50.into(),
            fee: None,
            preferred_providers: vec![],
        };

        let request = SwapperQuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(SUI_USDC_TOKEN_ID.to_string()))),
            wallet_address: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".to_string(),
            destination_address: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".to_string(),
            value: "1500000000".to_string(),
            mode: SwapperMode::ExactIn,
            options,
        };

        let quote = provider.fetch_quote(&request).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);
        assert_eq!(quote.data.slippage_bps, 50);

        let route = &quote.data.routes[0];
        assert_eq!(route.input, AssetId::from_chain(Chain::Sui));
        assert_eq!(route.output, AssetId::from(Chain::Sui, Some(SUI_USDC_TOKEN_ID.to_string())));
        assert!(!route.route_data.is_empty());

        Ok(())
    }
}
