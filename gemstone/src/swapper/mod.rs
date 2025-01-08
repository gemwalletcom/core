use crate::network::AlienProvider;

use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};

mod approval;
mod chainlink;
mod custom_types;
mod eth_rpc;
mod permit2_data;
mod weth_address;

pub mod across;
pub mod asset;
pub mod jupiter;
pub mod models;
pub mod orca;
pub mod pancakeswap_aptos;
pub mod slippage;
pub mod thorchain;
pub mod universal_router;

pub use models::*;
use primitives::{AssetId, Chain};
use std::collections::HashSet;

#[async_trait]
pub trait GemSwapProvider: Send + Sync + Debug {
    fn provider(&self) -> SwapProvider;
    fn supported_assets(&self) -> Vec<SwapChainAsset>;
    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError>;
    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError>;
    async fn get_transaction_status(&self, chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError>;
}

impl dyn GemSwapProvider {
    fn supported_chains(&self) -> Vec<Chain> {
        self.supported_assets()
            .into_iter()
            .map(|x| match x.clone() {
                SwapChainAsset::All(chain) => chain,
                SwapChainAsset::Assets(chain, _) => chain,
            })
            .collect()
    }
}

#[derive(Debug, uniffi::Object)]
pub struct GemSwapper {
    pub rpc_provider: Arc<dyn AlienProvider>,
    pub swappers: Vec<Box<dyn GemSwapProvider>>,
}

impl GemSwapper {
    // filter provider types that does not support cross chain / bridge swaps
    fn filter_by_provider_type(provider_type: SwapProviderType, from_chain: &Chain, to_chain: &Chain) -> bool {
        match provider_type {
            SwapProviderType::OnChain => from_chain == to_chain,
            SwapProviderType::CrossChain => true,
            SwapProviderType::Bridge => from_chain != to_chain,
        }
    }

    fn filter_by_supported_chains(supported_chains: Vec<Chain>, from_chain: &Chain, to_chain: &Chain) -> bool {
        supported_chains.contains(from_chain) && supported_chains.contains(to_chain)
    }

    fn filter_supported_assets(supported_assets: Vec<SwapChainAsset>, asset_id: AssetId) -> bool {
        supported_assets.into_iter().any(|x| match x {
            SwapChainAsset::All(_) => false,
            SwapChainAsset::Assets(chain, assets) => chain == asset_id.chain || assets.contains(&asset_id),
        })
    }
}

#[uniffi::export]
impl GemSwapper {
    #[uniffi::constructor]
    fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            rpc_provider,
            swappers: vec![
                Box::new(universal_router::UniswapV3::new_uniswap()),
                Box::new(universal_router::UniswapV3::new_pancakeswap()),
                Box::new(thorchain::ThorChain::default()),
                Box::new(jupiter::Jupiter::default()),
                Box::new(pancakeswap_aptos::PancakeSwapAptos::default()),
                Box::new(across::Across::default()),
            ],
        }
    }

    fn supported_chains(&self) -> Vec<Chain> {
        self.swappers
            .iter()
            .flat_map(|x| x.supported_chains())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    fn supported_chains_for_from_asset(&self, asset_id: AssetId) -> SwapAssetList {
        let chains: Vec<Chain> = vec![asset_id.chain];
        let mut asset_ids: Vec<AssetId> = Vec::new();

        for provider in &self.swappers {
            if !Self::filter_supported_assets(provider.supported_assets(), asset_id.clone()) {
                continue;
            }
            provider.supported_assets().into_iter().for_each(|x| match x {
                SwapChainAsset::All(_) => {}
                SwapChainAsset::Assets(chain, assets) => {
                    asset_ids.push(chain.as_asset_id());
                    asset_ids.extend(assets);
                }
            });
        }
        SwapAssetList { chains, asset_ids }
    }

    fn get_providers(&self) -> Vec<SwapProvider> {
        self.swappers.iter().map(|x| x.provider()).collect()
    }

    async fn fetch_quote(&self, request: SwapQuoteRequest) -> Result<Vec<SwapQuote>, SwapperError> {
        if request.from_asset == request.to_asset {
            return Err(SwapperError::NotSupportedPair);
        }
        let from_chain = request.from_asset.chain;
        let to_chain = request.to_asset.chain;

        let providers = self
            .swappers
            .iter()
            .filter(|x| Self::filter_by_provider_type(x.provider().provider_type(), &from_chain, &to_chain))
            .filter(|x| Self::filter_by_supported_chains(x.supported_chains(), &from_chain, &to_chain))
            .collect::<Vec<_>>();

        if providers.is_empty() {
            return Err(SwapperError::NoAvailableProvider);
        }

        let quotes_futures = providers.into_iter().map(|x| x.fetch_quote(&request, self.rpc_provider.clone()));

        let quotes = futures::future::join_all(quotes_futures.into_iter().map(|fut| async { fut.await.ok() }))
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        if quotes.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        Ok(quotes)
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let swapper = self
            .swappers
            .iter()
            .find(|x| x.provider() == quote.data.provider)
            .ok_or(SwapperError::NotImplemented)?;
        swapper.fetch_quote_data(quote, self.rpc_provider.clone(), data).await
    }

    async fn get_transaction_status(&self, chain: Chain, swap_provider: SwapProvider, transaction_hash: &str) -> Result<bool, SwapperError> {
        let swapper = self
            .swappers
            .iter()
            .find(|x| x.provider() == swap_provider)
            .ok_or(SwapperError::NotImplemented)?;

        swapper.get_transaction_status(chain, transaction_hash, self.rpc_provider.clone()).await
    }
}

#[cfg(test)]
mod tests {

    use asset::ETHEREUM_USDT_TOKEN_ID;

    use super::*;
    use std::{collections::BTreeSet, vec};

    #[test]
    fn test_filter_by_provider_type() {
        let providers = [
            SwapProvider::UniswapV3,
            SwapProvider::PancakeSwapV3,
            SwapProvider::Jupiter,
            SwapProvider::Thorchain,
        ];

        // Cross chain swaps (same chain will be filtered out)
        let filtered = providers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_type(x.provider_type(), &Chain::Ethereum, &Chain::Optimism))
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(filtered, vec![SwapProvider::Thorchain]);
    }

    #[test]
    fn test_filter_by_supported_chains() {
        let swappers: Vec<Box<dyn GemSwapProvider>> = vec![
            Box::new(universal_router::UniswapV3::new_uniswap()),
            Box::new(universal_router::UniswapV3::new_pancakeswap()),
            Box::new(thorchain::ThorChain::default()),
            Box::new(jupiter::Jupiter::default()),
        ];

        let from_chain = Chain::Ethereum;
        let to_chain = Chain::Optimism;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_type(x.provider().provider_type(), &from_chain, &to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), &from_chain, &to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 0);

        let from_chain = Chain::SmartChain;
        let to_chain = Chain::SmartChain;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_type(x.provider().provider_type(), &from_chain, &to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), &from_chain, &to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 3);
        assert_eq!(
            filtered.iter().map(|x| x.provider()).collect::<BTreeSet<_>>(),
            BTreeSet::from([SwapProvider::UniswapV3, SwapProvider::PancakeSwapV3, SwapProvider::Thorchain])
        );

        let from_chain = Chain::Solana;
        let to_chain = Chain::Solana;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_type(x.provider().provider_type(), &from_chain, &to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), &from_chain, &to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider(), SwapProvider::Jupiter);

        let from_chain = Chain::SmartChain;
        let to_chain = Chain::Bitcoin;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_type(x.provider().provider_type(), &from_chain, &to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), &from_chain, &to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider(), SwapProvider::Thorchain);
    }

    #[test]
    fn test_filter_supported_assets() {
        let asset_id = AssetId::from_chain(Chain::Ethereum);
        let asset_id_usdt = AssetId::from_token(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID);
        let supported_assets = vec![
            SwapChainAsset::All(Chain::Ethereum),
            SwapChainAsset::Assets(
                Chain::Ethereum,
                vec![AssetId::from_token(Chain::Ethereum, &asset_id_usdt.clone().token_id.unwrap())],
            ),
        ];

        assert!(GemSwapper::filter_supported_assets(supported_assets.clone(), asset_id_usdt.clone()));
        assert!(GemSwapper::filter_supported_assets(supported_assets, asset_id));
    }
}
