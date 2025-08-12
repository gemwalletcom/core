use crate::{debug_println, network::AlienProvider};

use async_trait::async_trait;
use num_traits::ToPrimitive;
use std::{fmt::Debug, sync::Arc};

mod approval;
mod chainlink;
mod custom_types;
mod eth_address;
mod permit2_data;

pub mod across;
pub mod asset;
pub mod cetus;
pub mod chainflip;
pub mod error;
pub mod jupiter;
pub mod models;
pub mod pancakeswap_aptos;
pub mod proxy;
pub mod remote_models;
pub mod slippage;
pub mod thorchain;
pub mod uniswap;

pub use error::*;
pub use models::*;
pub use remote_models::*;

use primitives::{AssetId, Chain, EVMChain};
use std::collections::HashSet;

#[async_trait]
pub trait Swapper: Send + Sync + Debug {
    fn provider(&self) -> &SwapperProviderType;
    fn supported_assets(&self) -> Vec<SwapperChainAsset>;
    async fn fetch_quote(&self, request: &SwapperQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapperQuote, SwapperError>;
    async fn fetch_permit2_for_quote(&self, _quote: &SwapperQuote, _provider: Arc<dyn AlienProvider>) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        Ok(None)
    }
    async fn fetch_quote_data(&self, quote: &SwapperQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError>;
    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        if self.provider().mode() == SwapperProviderMode::OnChain {
            Ok(true)
        } else {
            Err(SwapperError::NotImplemented)
        }
    }
}

impl dyn Swapper {
    fn supported_chains(&self) -> Vec<Chain> {
        self.supported_assets()
            .into_iter()
            .map(|x| match x.clone() {
                SwapperChainAsset::All(chain) => chain,
                SwapperChainAsset::Assets(chain, _) => chain,
            })
            .collect()
    }
}

#[derive(Debug, uniffi::Object)]
pub struct GemSwapper {
    pub rpc_provider: Arc<dyn AlienProvider>,
    pub swappers: Vec<Box<dyn Swapper>>,
}

impl GemSwapper {
    // filter provider types that does not support cross chain / bridge swaps
    fn filter_by_provider_mode(mode: SwapperProviderMode, from_chain: Chain, to_chain: Chain) -> bool {
        match mode {
            SwapperProviderMode::OnChain => from_chain == to_chain,
            SwapperProviderMode::Bridge | SwapperProviderMode::CrossChain => from_chain != to_chain,
            SwapperProviderMode::OmniChain(chains) => chains.contains(&from_chain) || from_chain != to_chain,
        }
    }

    fn filter_by_supported_chains(supported_chains: Vec<Chain>, from_chain: Chain, to_chain: Chain) -> bool {
        supported_chains.contains(&from_chain) && supported_chains.contains(&to_chain)
    }

    fn filter_supported_assets(supported_assets: Vec<SwapperChainAsset>, asset_id: AssetId) -> bool {
        supported_assets.into_iter().any(|x| match x {
            SwapperChainAsset::All(_) => false,
            SwapperChainAsset::Assets(chain, assets) => chain == asset_id.chain || assets.contains(&asset_id),
        })
    }

    fn filter_by_preferred_providers(preferred_providers: &[SwapperProvider], provider: &SwapperProvider) -> bool {
        // if no preferred providers, return all
        if preferred_providers.is_empty() {
            return true;
        }
        preferred_providers.contains(provider)
    }

    fn get_swapper_by_provider<'a>(&'a self, provider: &SwapperProvider) -> Option<&'a dyn Swapper> {
        self.swappers.iter().find(|x| x.provider().id == *provider).map(|v| &**v)
    }

    fn apply_gas_limit_multiplier(chain: &Chain, gas_limit: String) -> String {
        if let Some(evm_chain) = EVMChain::from_chain(*chain) {
            let multiplier = if evm_chain.is_zkstack() { 2.0 } else { 1.0 };
            if let Ok(gas_limit_value) = gas_limit.parse::<f64>() {
                return (gas_limit_value * multiplier).ceil().to_u64().unwrap_or_default().to_string();
            }
        }
        gas_limit
    }
}

#[uniffi::export]
impl GemSwapper {
    #[uniffi::constructor]
    pub fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            rpc_provider,
            swappers: vec![
                Box::new(uniswap::universal_router::new_uniswap_v3()),
                Box::new(uniswap::universal_router::new_uniswap_v4()),
                Box::new(uniswap::universal_router::new_pancakeswap()),
                Box::new(thorchain::ThorChain::default()),
                Box::new(jupiter::Jupiter::default()),
                Box::new(across::Across::default()),
                Box::new(uniswap::universal_router::new_oku()),
                Box::new(uniswap::universal_router::new_wagmi()),
                Box::new(uniswap::universal_router::new_reservoir()),
                Box::new(pancakeswap_aptos::PancakeSwapAptos::default()),
                Box::new(proxy::new_stonfi_v2()),
                Box::new(proxy::new_mayan()),
                Box::new(chainflip::ChainflipProvider::default()),
                Box::new(proxy::new_cetus_aggregator()),
                Box::new(proxy::new_relay()),
                Box::new(uniswap::universal_router::new_aerodrome()),
            ],
        }
    }

    pub fn supported_chains(&self) -> Vec<Chain> {
        self.swappers
            .iter()
            .flat_map(|x| x.supported_chains())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn supported_chains_for_from_asset(&self, asset_id: &AssetId) -> SwapperAssetList {
        let chains: Vec<Chain> = vec![asset_id.chain];
        let mut asset_ids: Vec<AssetId> = Vec::new();

        for provider in &self.swappers {
            if !Self::filter_supported_assets(provider.supported_assets(), asset_id.clone()) {
                continue;
            }
            provider.supported_assets().into_iter().for_each(|x| match x {
                SwapperChainAsset::All(_) => {}
                SwapperChainAsset::Assets(chain, assets) => {
                    asset_ids.push(chain.as_asset_id());
                    asset_ids.extend(assets);
                }
            });
        }
        SwapperAssetList { chains, asset_ids }
    }

    pub fn get_providers(&self) -> Vec<SwapperProviderType> {
        self.swappers.iter().map(|x| x.provider().clone()).collect()
    }

    pub async fn fetch_quote(&self, request: &SwapperQuoteRequest) -> Result<Vec<SwapperQuote>, SwapperError> {
        if request.from_asset.id == request.to_asset.id {
            return Err(SwapperError::NotSupportedPair);
        }
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        let preferred_providers = &request.options.preferred_providers;
        let providers = self
            .swappers
            .iter()
            .filter(|x| Self::filter_by_provider_mode(x.provider().mode(), from_chain, to_chain))
            .filter(|x| Self::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .filter(|x| Self::filter_by_preferred_providers(preferred_providers, &x.provider().id))
            .collect::<Vec<_>>();

        if providers.is_empty() {
            return Err(SwapperError::NoAvailableProvider);
        }

        let quotes_futures = providers.into_iter().map(|x| x.fetch_quote(request, self.rpc_provider.clone()));

        let quotes = futures::future::join_all(quotes_futures.into_iter().map(|fut| async {
            match &fut.await {
                Ok(quote) => Some(quote.clone()),
                Err(_err) => {
                    debug_println!("fetch_quote error: {:?}", _err);
                    None
                }
            }
        }))
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        if quotes.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        Ok(quotes)
    }

    pub async fn fetch_quote_by_provider(&self, provider: SwapperProvider, request: SwapperQuoteRequest) -> Result<SwapperQuote, SwapperError> {
        let provider = self.get_swapper_by_provider(&provider).ok_or(SwapperError::NoAvailableProvider)?;
        provider.fetch_quote(&request, self.rpc_provider.clone()).await
    }

    pub async fn fetch_permit2_for_quote(&self, quote: &SwapperQuote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let provider = self.get_swapper_by_provider(&quote.data.provider.id).ok_or(SwapperError::NoAvailableProvider)?;
        provider.fetch_permit2_for_quote(quote, self.rpc_provider.clone()).await
    }

    pub async fn fetch_quote_data(&self, quote: &SwapperQuote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let provider = self.get_swapper_by_provider(&quote.data.provider.id).ok_or(SwapperError::NoAvailableProvider)?;
        let mut quote_data = provider.fetch_quote_data(quote, self.rpc_provider.clone(), data).await?;
        if let Some(gas_limit) = quote_data.gas_limit.take() {
            quote_data.gas_limit = Some(Self::apply_gas_limit_multiplier(&quote.request.from_asset.chain(), gas_limit));
        }
        Ok(quote_data)
    }

    pub async fn get_transaction_status(&self, chain: Chain, swap_provider: SwapperProvider, transaction_hash: &str) -> Result<bool, SwapperError> {
        let provider = self.get_swapper_by_provider(&swap_provider).ok_or(SwapperError::NoAvailableProvider)?;
        provider.get_transaction_status(chain, transaction_hash, self.rpc_provider.clone()).await
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use primitives::asset_constants::USDT_ETH_ASSET_ID;
    use std::{collections::BTreeSet, vec};

    #[test]
    fn test_filter_by_provider_type() {
        let providers = [
            SwapperProvider::UniswapV3,
            SwapperProvider::PancakeswapV3,
            SwapperProvider::Jupiter,
            SwapperProvider::Thorchain,
        ];

        // Cross chain swaps (same chain will be filtered out)
        let filtered = providers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_mode(SwapperProviderType::new(**x).mode(), Chain::Ethereum, Chain::Optimism))
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(filtered, vec![SwapperProvider::Thorchain]);
    }

    #[test]
    fn test_filter_by_supported_chains() {
        let swappers: Vec<Box<dyn Swapper>> = vec![
            Box::new(uniswap::universal_router::new_uniswap_v3()),
            Box::new(uniswap::universal_router::new_pancakeswap()),
            Box::new(thorchain::ThorChain::default()),
            Box::new(jupiter::Jupiter::default()),
        ];

        let from_chain = Chain::Ethereum;
        let to_chain = Chain::Optimism;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_mode(x.provider().mode(), from_chain, to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 0);

        let from_chain = Chain::SmartChain;
        let to_chain = Chain::SmartChain;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_mode(x.provider().mode(), from_chain, to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 2);
        assert_eq!(
            filtered.iter().map(|x| x.provider().id).collect::<BTreeSet<_>>(),
            BTreeSet::from([SwapperProvider::UniswapV3, SwapperProvider::PancakeswapV3])
        );

        let from_chain = Chain::Solana;
        let to_chain = Chain::Solana;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_mode(x.provider().mode(), from_chain, to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider().id, SwapperProvider::Jupiter);

        let from_chain = Chain::SmartChain;
        let to_chain = Chain::Bitcoin;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_mode(x.provider().mode(), from_chain, to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider().id, SwapperProvider::Thorchain);
    }

    #[test]
    fn test_filter_supported_assets() {
        let asset_id = AssetId::from_chain(Chain::Ethereum);
        let asset_id_usdt: AssetId = USDT_ETH_ASSET_ID.into();
        let supported_assets = vec![
            SwapperChainAsset::All(Chain::Ethereum),
            SwapperChainAsset::Assets(
                Chain::Ethereum,
                vec![AssetId::from_token(Chain::Ethereum, &asset_id_usdt.clone().token_id.unwrap())],
            ),
        ];

        assert!(GemSwapper::filter_supported_assets(supported_assets.clone(), asset_id_usdt.clone()));
        assert!(GemSwapper::filter_supported_assets(supported_assets, asset_id));
    }
}
