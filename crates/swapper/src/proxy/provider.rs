use alloy_primitives::U256;
use async_trait::async_trait;
use std::{fmt::Debug, str::FromStr, sync::Arc};

use super::client::ProxyClient;
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapResult, Swapper, SwapperError, SwapperProvider, SwapperProviderMode, SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    approval::{DEFAULT_EVM_SWAP_GAS_LIMIT, check_approval_erc20, get_swap_gas_limit_with_approval},
    config::get_swap_proxy_url,
    cross_chain::VaultAddresses,
    fees::{DEFAULT_AGGREGATOR_FEE_BPS, DEFAULT_SWAP_FEE_BPS},
    models::SwapperChainAsset,
};
use gem_client::Client;
use primitives::{
    Chain, ChainType,
    swap::{ApprovalData, ProxyQuote, ProxyQuoteRequest, SwapQuoteData},
};

#[derive(Debug)]
pub struct ProxyProvider<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub provider: ProviderType,
    pub assets: Vec<SwapperChainAsset>,
    client: ProxyClient<C>,
    pub(crate) rpc_provider: Arc<dyn RpcProvider>,
}

impl<C> ProxyProvider<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn new_with_client(provider: SwapperProvider, client: ProxyClient<C>, assets: Vec<SwapperChainAsset>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(provider),
            assets,
            client,
            rpc_provider,
        }
    }

    pub async fn check_approval_and_limit(&self, quote: &Quote, quote_data: &SwapQuoteData) -> Result<(Option<ApprovalData>, Option<String>), SwapperError> {
        if let Some(ref approval) = quote_data.approval {
            let approval = Some(approval.clone());
            let gas_limit = get_swap_gas_limit_with_approval(&approval, quote_data.gas_limit.clone(), DEFAULT_EVM_SWAP_GAS_LIMIT);
            return Ok((approval, gas_limit));
        }

        let request = &quote.request;
        let from_asset = request.from_asset.asset_id();

        match from_asset.chain.chain_type() {
            ChainType::Ethereum => {
                if from_asset.is_native() {
                    Ok((None, None))
                } else {
                    let token = from_asset.token_id.ok_or(SwapperError::NotSupportedAsset)?;
                    self.check_evm_approval(
                        request.wallet_address.clone(),
                        token,
                        quote_data.to.clone(),
                        U256::from_str(&quote.from_value).map_err(SwapperError::from)?,
                        &from_asset.chain,
                        quote_data.gas_limit.clone(),
                    )
                    .await
                }
            }
            _ => Ok((None, quote_data.gas_limit.clone())),
        }
    }

    async fn check_evm_approval(
        &self,
        wallet_address: String,
        token: String,
        spender: String,
        amount: U256,
        chain: &Chain,
        swap_gas_limit: Option<String>,
    ) -> Result<(Option<ApprovalData>, Option<String>), SwapperError> {
        let approval = check_approval_erc20(wallet_address, token, spender, amount, self.rpc_provider.clone(), chain).await?;
        let approval = approval.approval_data();
        let gas_limit = get_swap_gas_limit_with_approval(&approval, swap_gas_limit, DEFAULT_EVM_SWAP_GAS_LIMIT);
        Ok((approval, gas_limit))
    }

    fn referral_bps(&self) -> u32 {
        match self.provider.id {
            SwapperProvider::Okx => DEFAULT_AGGREGATOR_FEE_BPS,
            _ => DEFAULT_SWAP_FEE_BPS,
        }
    }
}

impl ProxyProvider<RpcClient> {
    fn new_with_path(provider: SwapperProvider, path: &str, assets: Vec<SwapperChainAsset>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let base_url = get_swap_proxy_url(&format!("swapper/{path}"));
        let client = ProxyClient::new(RpcClient::new(base_url, rpc_provider.clone()));
        Self::new_with_client(provider, client, assets, rpc_provider)
    }

    pub fn new_okx(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_path(
            SwapperProvider::Okx,
            "okx",
            vec![
                SwapperChainAsset::All(Chain::Solana),
                SwapperChainAsset::All(Chain::Ethereum),
                SwapperChainAsset::All(Chain::SmartChain),
                SwapperChainAsset::All(Chain::Polygon),
                SwapperChainAsset::All(Chain::Arbitrum),
                SwapperChainAsset::All(Chain::Optimism),
                SwapperChainAsset::All(Chain::Base),
                SwapperChainAsset::All(Chain::AvalancheC),
                SwapperChainAsset::All(Chain::OpBNB),
                SwapperChainAsset::All(Chain::Fantom),
                SwapperChainAsset::All(Chain::Gnosis),
                SwapperChainAsset::All(Chain::Manta),
                SwapperChainAsset::All(Chain::Blast),
                SwapperChainAsset::All(Chain::ZkSync),
                SwapperChainAsset::All(Chain::Linea),
                SwapperChainAsset::All(Chain::Mantle),
                SwapperChainAsset::All(Chain::Celo),
                SwapperChainAsset::All(Chain::Sonic),
                SwapperChainAsset::All(Chain::Abstract),
                SwapperChainAsset::All(Chain::Berachain),
                SwapperChainAsset::All(Chain::Unichain),
                SwapperChainAsset::All(Chain::Monad),
                SwapperChainAsset::All(Chain::XLayer),
            ],
            rpc_provider,
        )
    }
}

#[async_trait]
impl<C> Swapper for ProxyProvider<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        self.assets.clone()
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let quote_request = ProxyQuoteRequest {
            from_address: request.wallet_address.clone(),
            to_address: request.destination_address.clone(),
            from_asset: request.from_asset.clone(),
            to_asset: request.to_asset.clone(),
            from_value: request.value.clone(),
            referral_bps: self.referral_bps(),
            slippage_bps: request.options.slippage.bps,
            use_max_amount: request.options.use_max_amount,
        };

        let quote = self.client.get_quote(quote_request.clone()).await?;

        Ok(Quote {
            from_value: request.value.clone(),
            to_value: quote.output_value.clone(),
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&quote).map_err(SwapperError::compute_quote_error)?,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(quote.eta_in_seconds),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data: ProxyQuote = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let data = self.client.get_quote_data(route_data).await?;
        let (approval, gas_limit) = self.check_approval_and_limit(quote, &data).await?;

        Ok(SwapperQuoteData::new_contract(data.to, data.value, data.data, approval, gas_limit))
    }

    async fn get_swap_result(&self, _chain: Chain, _transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        if self.provider.mode == SwapperProviderMode::OnChain {
            Ok(SwapResult {
                status: primitives::swap::SwapStatus::Completed,
                metadata: None,
            })
        } else {
            Err(SwapperError::NotSupportedAsset)
        }
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        Ok(VaultAddresses { deposit: vec![], send: vec![] })
    }
}

#[cfg(test)]
mod tests {
    use super::super::client::ProxyClient;
    use super::*;
    use crate::{
        alien::mock::ProviderMock,
        fees::{DEFAULT_AGGREGATOR_FEE_BPS, DEFAULT_SWAP_FEE_BPS},
    };
    use gem_client::testkit::MockClient;
    use primitives::swap::{ApprovalData, SwapQuoteData};

    fn mock_provider(provider: SwapperProvider) -> ProxyProvider<MockClient> {
        let rpc_provider = Arc::new(ProviderMock::new("{}".to_string()));
        ProxyProvider::new_with_client(provider, ProxyClient::new(MockClient::new()), vec![], rpc_provider)
    }

    #[test]
    fn test_referral_bps() {
        assert_eq!(mock_provider(SwapperProvider::Okx).referral_bps(), DEFAULT_AGGREGATOR_FEE_BPS);
        assert_eq!(mock_provider(SwapperProvider::StonfiV2).referral_bps(), DEFAULT_SWAP_FEE_BPS);
    }

    #[tokio::test]
    async fn test_solana_preserves_provider_gas_limit() {
        let provider = mock_provider(SwapperProvider::Okx);
        let quote = Quote::mock(Chain::Solana, None);
        let data = SwapQuoteData::mock_with_gas_limit(Some("550000".to_string()));

        let (approval, gas_limit) = provider.check_approval_and_limit(&quote, &data).await.unwrap();

        assert!(approval.is_none());
        assert_eq!(gas_limit, Some("550000".to_string()));

        let data = SwapQuoteData::mock_with_gas_limit(None);

        let (approval, gas_limit) = provider.check_approval_and_limit(&quote, &data).await.unwrap();

        assert!(approval.is_none());
        assert!(gas_limit.is_none());
    }

    #[tokio::test]
    async fn test_evm_native_ignores_provider_gas_limit() {
        let provider = mock_provider(SwapperProvider::Okx);
        let quote = Quote::mock(Chain::Ethereum, None);
        let data = SwapQuoteData::mock_with_gas_limit(Some("550000".to_string()));

        let (approval, gas_limit) = provider.check_approval_and_limit(&quote, &data).await.unwrap();

        assert!(approval.is_none());
        assert!(gas_limit.is_none());
    }

    #[tokio::test]
    async fn test_evm_provider_approval_uses_swap_gas_limit() {
        let provider = mock_provider(SwapperProvider::Okx);
        let quote = Quote::mock(Chain::Ethereum, None);

        let data = SwapQuoteData {
            approval: Some(ApprovalData::mock()),
            ..SwapQuoteData::mock_with_gas_limit(Some("250000".to_string()))
        };
        let (approval, gas_limit) = provider.check_approval_and_limit(&quote, &data).await.unwrap();
        assert!(approval.is_some());
        assert_eq!(gas_limit, Some("250000".to_string()));

        let data = SwapQuoteData {
            approval: Some(ApprovalData::mock()),
            ..SwapQuoteData::mock_with_gas_limit(None)
        };
        let (approval, gas_limit) = provider.check_approval_and_limit(&quote, &data).await.unwrap();
        assert!(approval.is_some());
        assert_eq!(gas_limit, Some(DEFAULT_EVM_SWAP_GAS_LIMIT.to_string()));
    }
}
