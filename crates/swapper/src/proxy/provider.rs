use alloy_primitives::U256;
use async_trait::async_trait;
use std::{fmt::Debug, str::FromStr, sync::Arc};

use super::{
    client::ProxyClient,
    mayan::{MayanClientStatus, MayanExplorer, wormhole_chain_id_to_chain},
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapResult, Swapper, SwapperError, SwapperProvider, SwapperProviderMode,
    SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    approval::evm::check_approval_erc20,
    asset::*,
    config::DEFAULT_SWAP_FEE_BPS,
    models::{ApprovalType, SwapperChainAsset},
};
use gem_client::Client;
use primitives::{
    Chain, ChainType,
    swap::{ApprovalData, ProxyQuote, ProxyQuoteRequest, SwapQuoteData},
};

pub const PROVIDER_API_URL: &str = "https://api.gemwallet.com/swap/swapper";
const DEFAULT_GAS_LIMIT: u64 = 750_000;

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

    pub async fn check_approval(&self, quote: &Quote, quote_data: &SwapQuoteData) -> Result<(Option<ApprovalData>, Option<String>), SwapperError> {
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
            ChainType::Tron => Ok((None, None)),
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
    ) -> Result<(Option<ApprovalData>, Option<String>), SwapperError> {
        let approval = check_approval_erc20(wallet_address, token, spender, amount, self.rpc_provider.clone(), chain).await?;
        let gas_limit = if matches!(approval, ApprovalType::Approve(_)) {
            Some(DEFAULT_GAS_LIMIT.to_string())
        } else {
            None
        };
        Ok((approval.approval_data(), gas_limit))
    }
}

impl ProxyProvider<RpcClient> {
    fn new_with_path(provider: SwapperProvider, path: &str, assets: Vec<SwapperChainAsset>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let base_url = format!("{PROVIDER_API_URL}/{path}");
        let client = ProxyClient::new(RpcClient::new(base_url, rpc_provider.clone()));
        Self::new_with_client(provider, client, assets, rpc_provider)
    }

    pub fn new_stonfi_v2(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_path(SwapperProvider::StonfiV2, "stonfi_v2", vec![SwapperChainAsset::All(Chain::Ton)], rpc_provider)
    }

    pub fn new_orca(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_path(SwapperProvider::Orca, "orca", vec![SwapperChainAsset::All(Chain::Solana)], rpc_provider)
    }

    pub fn new_cetus_aggregator(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_path(
            SwapperProvider::CetusAggregator,
            "cetus",
            vec![SwapperChainAsset::All(Chain::Sui)],
            rpc_provider,
        )
    }

    pub fn new_mayan(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let assets = vec![
            SwapperChainAsset::Assets(
                Chain::Ethereum,
                vec![
                    ETHEREUM_USDT.id.clone(),
                    ETHEREUM_USDC.id.clone(),
                    ETHEREUM_DAI.id.clone(),
                    ETHEREUM_USDS.id.clone(),
                    ETHEREUM_WBTC.id.clone(),
                    ETHEREUM_WETH.id.clone(),
                    ETHEREUM_STETH.id.clone(),
                    ETHEREUM_CBBTC.id.clone(),
                ],
            ),
            SwapperChainAsset::Assets(
                Chain::Solana,
                vec![
                    SOLANA_USDC.id.clone(),
                    SOLANA_USDT.id.clone(),
                    SOLANA_USDS.id.clone(),
                    SOLANA_CBBTC.id.clone(),
                    SOLANA_WBTC.id.clone(),
                    SOLANA_JITO_SOL.id.clone(),
                ],
            ),
            SwapperChainAsset::Assets(Chain::Sui, vec![SUI_USDC.id.clone(), SUI_SBUSDT.id.clone(), SUI_WAL.id.clone()]),
            SwapperChainAsset::Assets(
                Chain::SmartChain,
                vec![SMARTCHAIN_USDT.id.clone(), SMARTCHAIN_USDC.id.clone(), SMARTCHAIN_WBTC.id.clone()],
            ),
            SwapperChainAsset::Assets(
                Chain::Base,
                vec![BASE_USDC.id.clone(), BASE_CBBTC.id.clone(), BASE_WBTC.id.clone(), BASE_USDS.id.clone()],
            ),
            SwapperChainAsset::Assets(Chain::Polygon, vec![POLYGON_USDC.id.clone(), POLYGON_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::AvalancheC, vec![AVALANCHE_USDT.id.clone(), AVALANCHE_USDC.id.clone()]),
            SwapperChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDC.id.clone(), ARBITRUM_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_USDC.id.clone(), OPTIMISM_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::Linea, vec![LINEA_USDC.id.clone(), LINEA_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::Unichain, vec![UNICHAIN_USDC.id.clone(), UNICHAIN_DAI.id.clone()]),
            SwapperChainAsset::Assets(Chain::Hyperliquid, vec![HYPEREVM_USDT.id.clone(), HYPEREVM_USDC.id.clone()]),
        ];

        Self::new_with_path(SwapperProvider::Mayan, "mayan", assets, rpc_provider)
    }

    pub fn new_relay(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_path(
            SwapperProvider::Relay,
            "relay",
            vec![
                SwapperChainAsset::All(Chain::Hyperliquid),
                SwapperChainAsset::All(Chain::Manta),
                SwapperChainAsset::All(Chain::Berachain),
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

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let quote_request = ProxyQuoteRequest {
            from_address: request.wallet_address.clone(),
            to_address: request.destination_address.clone(),
            from_asset: request.from_asset.clone(),
            to_asset: request.to_asset.clone(),
            from_value: request.value.clone(),
            referral_bps: DEFAULT_SWAP_FEE_BPS,
            slippage_bps: request.options.slippage.bps,
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
                    route_data: serde_json::to_string(&quote).unwrap(),
                    gas_limit: None,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(quote.eta_in_seconds),
        })
    }

    async fn fetch_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let routes = quote.data.clone().routes;
        let route_data: ProxyQuote = serde_json::from_str(&routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let data = self.client.get_quote_data(route_data).await?;
        let (approval, gas_limit) = self.check_approval(quote, &data).await?;

        Ok(SwapperQuoteData::new_contract(data.to, data.value, data.data, approval, gas_limit))
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        match self.provider.id {
            SwapperProvider::Mayan => {
                let client = MayanExplorer::new(self.rpc_provider.clone());
                let result = client.get_transaction_status(transaction_hash).await?;

                let swap_status = result.client_status.swap_status();
                let dest_chain = result.dest_chain.parse::<u16>().ok().and_then(wormhole_chain_id_to_chain);

                let (to_chain, to_tx_hash) = match result.client_status {
                    MayanClientStatus::Completed => (dest_chain, result.fulfill_tx_hash),
                    MayanClientStatus::Refunded | MayanClientStatus::InProgress => (dest_chain, None),
                };

                Ok(SwapResult {
                    status: swap_status,
                    from_chain: chain,
                    from_tx_hash: transaction_hash.to_string(),
                    to_chain,
                    to_tx_hash,
                })
            }
            // For OnChain providers, use the default implementation
            _ => {
                if self.provider.mode == SwapperProviderMode::OnChain {
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
        alien::reqwest_provider::NativeProvider,
        {SwapperMode, SwapperQuoteAsset, asset::SUI_USDC_TOKEN_ID, models::Options},
    };
    use primitives::AssetId;

    #[tokio::test]
    async fn test_mayan_provider_fetch_quote() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = ProxyProvider::new_mayan(rpc_provider);

        let options = Options::new_with_slippage(200.into());

        let request = QuoteRequest {
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

        let options = Options::new_with_slippage(50.into());

        let request = QuoteRequest {
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

    #[tokio::test]
    #[cfg(feature = "swap_integration_tests")]
    async fn test_mayan_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use primitives::swap::SwapStatus;

        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = ProxyProvider::new_mayan(rpc_provider);

        // Real Mayan swap: ETH to SUI via CCTP
        // Ethereum source tx: 0x56acc6a58fc0bdd9e9be5cc2a3ff079b91b933f562cf0fe760f1d8d6b76f4876
        let tx_hash = "0x56acc6a58fc0bdd9e9be5cc2a3ff079b91b933f562cf0fe760f1d8d6b76f4876";
        let chain = Chain::Ethereum;

        let result = provider.get_swap_result(chain, tx_hash).await?;

        println!("Mayan swap result: {:?}", result);
        assert_eq!(result.from_chain, chain);
        assert_eq!(result.from_tx_hash, tx_hash);
        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(result.to_chain, Some(Chain::Sui));
        assert_eq!(result.to_tx_hash, Some("GLs1TUZ6jQdWBBDHVBYFumaBMf6kVNcb2NxQnapXqJUL".to_string()));

        Ok(())
    }
}
