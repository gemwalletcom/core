use crate::{
    AssetList, FetchQuoteData, Permit2ApprovalData, ProviderType, Quote, QuoteRequest, SwapQuoteError, SwapQuotes, SwapResult, Swapper, SwapperChainAsset, SwapperError,
    SwapperProvider, SwapperProviderMode, SwapperQuoteData, across, alien::RpcProvider, cetus_clmm, chainflip, cross_chain::VaultAddresses, hyperliquid, jupiter, mayan,
    near_intents, panora, proxy::provider_factory, relay, squid, stonfi, thorchain, uniswap,
};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use primitives::{AssetId, Chain, EVMChain};
use std::{
    collections::{BTreeSet, HashSet},
    fmt::Debug,
    sync::Arc,
};

#[derive(Debug)]
pub struct GemSwapper {
    pub rpc_provider: Arc<dyn RpcProvider>,
    pub swappers: Vec<Box<dyn Swapper>>,
}

impl GemSwapper {
    // filter provider types that does not support cross chain / bridge swaps
    fn filter_by_provider_mode(mode: &SwapperProviderMode, from_chain: Chain, to_chain: Chain) -> bool {
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
            SwapperChainAsset::All(chain) => chain == asset_id.chain,
            SwapperChainAsset::Assets(chain, assets) => chain == asset_id.chain || assets.contains(&asset_id),
        })
    }

    fn get_swapper_by_provider(&self, provider: &SwapperProvider) -> Result<&dyn Swapper, SwapperError> {
        self.swappers
            .iter()
            .find(|x| x.provider().id == *provider)
            .map(|v| &**v)
            .ok_or(SwapperError::NoAvailableProvider)
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

    fn sort_quotes_by_output_amount(quotes: &mut [Quote]) {
        quotes.sort_by(Self::compare_quotes_by_output_amount);
    }

    fn compare_quotes_by_output_amount(a: &Quote, b: &Quote) -> std::cmp::Ordering {
        let a_amount = a.to_value.parse::<BigInt>().unwrap_or_default();
        let b_amount = b.to_value.parse::<BigInt>().unwrap_or_default();
        b_amount.cmp(&a_amount)
    }
}

impl GemSwapper {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let swappers: Vec<Box<dyn Swapper>> = vec![
            uniswap::default::boxed_uniswap_v3(rpc_provider.clone()),
            uniswap::default::boxed_uniswap_v4(rpc_provider.clone()),
            uniswap::default::boxed_pancakeswap(rpc_provider.clone()),
            Box::new(thorchain::ThorChain::new(rpc_provider.clone())),
            Box::new(jupiter::Jupiter::new(rpc_provider.clone())),
            Box::new(provider_factory::new_okx(rpc_provider.clone())),
            Box::new(across::Across::new(rpc_provider.clone())),
            Box::new(hyperliquid::Hyperliquid::new(rpc_provider.clone())),
            uniswap::default::boxed_oku(rpc_provider.clone()),
            uniswap::default::boxed_wagmi(rpc_provider.clone()),
            Box::new(stonfi::Stonfi::new(rpc_provider.clone())),
            Box::new(mayan::Mayan::new(rpc_provider.clone())),
            Box::new(panora::Panora::new(rpc_provider.clone())),
            Box::new(near_intents::NearIntents::new(rpc_provider.clone())),
            Box::new(chainflip::ChainflipProvider::new(rpc_provider.clone())),
            Box::new(cetus_clmm::CetusClmm::new(rpc_provider.clone())),
            Box::new(relay::Relay::new(rpc_provider.clone())),
            Box::new(squid::Squid::new(rpc_provider.clone())),
            uniswap::default::boxed_aerodrome(rpc_provider.clone()),
        ];

        Self { rpc_provider, swappers }
    }

    pub fn supported_chains(&self) -> Vec<Chain> {
        self.swappers.iter().flat_map(|x| x.supported_chains()).collect::<HashSet<_>>().into_iter().collect()
    }

    pub fn supported_chains_for_from_asset(&self, asset_id: &AssetId) -> AssetList {
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
        AssetList { chains, asset_ids }
    }

    pub fn get_providers(&self) -> Vec<ProviderType> {
        self.swappers.iter().map(|x| x.provider().clone()).collect()
    }

    pub fn get_providers_for_request(&self, request: &QuoteRequest) -> Result<Vec<ProviderType>, SwapperError> {
        if request.from_asset.id == request.to_asset.id {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        let providers: Vec<ProviderType> = self
            .swappers
            .iter()
            .filter(|x| Self::filter_by_provider_mode(&x.provider().mode, from_chain, to_chain))
            .filter(|x| Self::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .map(|x| x.provider().clone())
            .collect();
        if providers.is_empty() {
            return Err(SwapperError::NoAvailableProvider);
        }
        Ok(providers)
    }

    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<Vec<Quote>, SwapperError> {
        let SwapQuotes { quotes, .. } = self.get_quotes(request).await?;
        if quotes.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }
        Ok(quotes)
    }

    pub async fn get_quotes(&self, request: &QuoteRequest) -> Result<SwapQuotes, SwapperError> {
        let provider_ids: BTreeSet<_> = self.get_providers_for_request(request)?.into_iter().map(|p| p.id).collect();
        let providers = self.swappers.iter().filter(|x| provider_ids.contains(&x.provider().id)).collect::<Vec<_>>();

        let quotes_futures = providers.into_iter().map(|x| {
            let provider_id = x.provider().id.id().to_string();
            async move { x.get_quote(request).await.map_err(|e| (provider_id, e)) }
        });

        let quote_results = futures::future::join_all(quotes_futures).await;

        let mut quotes = Vec::new();
        let mut errors = Vec::new();
        for result in quote_results {
            match result {
                Ok(quote) => quotes.push(quote),
                Err((provider_id, err)) => errors.push(SwapQuoteError::new(Some(provider_id), err.to_string())),
            }
        }

        Self::sort_quotes_by_output_amount(&mut quotes);
        Ok(SwapQuotes { quotes, errors })
    }

    pub async fn get_quote_by_provider(&self, provider: SwapperProvider, request: QuoteRequest) -> Result<Quote, SwapperError> {
        let provider = self.get_swapper_by_provider(&provider)?;
        provider.get_quote(&request).await
    }

    pub async fn get_permit2_for_quote(&self, quote: &Quote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let provider = self.get_swapper_by_provider(&quote.data.provider.id)?;
        provider.get_permit2_for_quote(quote).await
    }

    pub async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let provider = self.get_swapper_by_provider(&quote.data.provider.id)?;
        let mut quote_data = provider.get_quote_data(quote, data).await?;
        if let Some(gas_limit) = quote_data.gas_limit.take() {
            quote_data.gas_limit = Some(Self::apply_gas_limit_multiplier(&quote.request.from_asset.chain(), gas_limit));
        }
        Ok(quote_data)
    }

    pub async fn get_swap_result(&self, chain: Chain, provider: SwapperProvider, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        self.get_swapper_by_provider(&provider)?.get_swap_result(chain, transaction_hash).await
    }

    pub async fn get_vault_addresses(&self, provider: &SwapperProvider, from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        self.get_swapper_by_provider(provider)?.get_vault_addresses(from_timestamp).await
    }
}

#[cfg(all(test, feature = "reqwest_provider"))]
mod tests {

    use std::{collections::BTreeSet, sync::Arc, vec};

    use primitives::{
        AssetId, Chain,
        asset_constants::{ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID},
    };

    use super::*;
    use crate::{
        SwapperChainAsset, SwapperProvider, SwapperQuoteAsset,
        alien::reqwest_provider::NativeProvider,
        testkit::{MockSwapper, mock_quote},
        uniswap::default::{new_pancakeswap, new_uniswap_v3},
    };

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
            .filter(|x| GemSwapper::filter_by_provider_mode(&ProviderType::new(**x).mode, Chain::Ethereum, Chain::Optimism))
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(filtered, vec![SwapperProvider::Thorchain]);
    }

    #[test]
    fn test_filter_by_supported_chains() {
        let provider = Arc::new(NativeProvider::default());
        let swappers: Vec<Box<dyn Swapper>> = vec![
            Box::new(new_uniswap_v3(provider.clone())),
            Box::new(new_pancakeswap(provider.clone())),
            Box::new(thorchain::ThorChain::new(provider.clone())),
            Box::new(jupiter::Jupiter::new(provider)),
        ];

        let from_chain = Chain::Ethereum;
        let to_chain = Chain::Optimism;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_mode(&x.provider().mode, from_chain, to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 0);

        let from_chain = Chain::SmartChain;
        let to_chain = Chain::SmartChain;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_mode(&x.provider().mode, from_chain, to_chain))
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
            .filter(|x| GemSwapper::filter_by_provider_mode(&x.provider().mode, from_chain, to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider().id, SwapperProvider::Jupiter);

        let from_chain = Chain::SmartChain;
        let to_chain = Chain::Bitcoin;

        let filtered = swappers
            .iter()
            .filter(|x| GemSwapper::filter_by_provider_mode(&x.provider().mode, from_chain, to_chain))
            .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider().id, SwapperProvider::Thorchain);
    }

    #[test]
    fn test_filter_supported_assets() {
        let asset_id = AssetId::from_chain(Chain::Ethereum);
        let asset_id_usdt: AssetId = ETHEREUM_USDT_ASSET_ID.clone();
        let supported_assets_all = vec![SwapperChainAsset::All(Chain::Ethereum)];
        assert!(GemSwapper::filter_supported_assets(supported_assets_all, asset_id.clone()));

        let supported_assets = vec![
            SwapperChainAsset::All(Chain::Ethereum),
            SwapperChainAsset::Assets(Chain::Ethereum, vec![AssetId::from_token(Chain::Ethereum, &asset_id_usdt.clone().token_id.unwrap())]),
        ];

        assert!(GemSwapper::filter_supported_assets(supported_assets.clone(), asset_id_usdt.clone()));
        assert!(GemSwapper::filter_supported_assets(supported_assets, asset_id));
    }

    #[tokio::test]
    async fn test_get_quotes_collects_per_provider_errors() {
        let request = mock_quote(
            SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
            SwapperQuoteAsset::from(ETHEREUM_USDC_ASSET_ID.clone()),
        );

        let gem_swapper = GemSwapper {
            rpc_provider: Arc::new(NativeProvider::default()),
            swappers: vec![
                Box::new(MockSwapper::new(SwapperProvider::UniswapV3, || Err(SwapperError::InputAmountError { min_amount: None }))),
                Box::new(MockSwapper::new(SwapperProvider::PancakeswapV3, || {
                    Err(SwapperError::InputAmountError {
                        min_amount: Some("1264000".into()),
                    })
                })),
                Box::new(MockSwapper::new(SwapperProvider::Jupiter, || Err(SwapperError::NoQuoteAvailable))),
            ],
        };
        let result = gem_swapper.get_quotes(&request).await.unwrap();
        assert!(result.quotes.is_empty());
        assert_eq!(result.errors.len(), 3);

        let providers: BTreeSet<_> = result.errors.iter().map(|e| e.provider.clone().unwrap()).collect();
        assert_eq!(
            providers,
            BTreeSet::from([
                SwapperProvider::UniswapV3.id().to_string(),
                SwapperProvider::PancakeswapV3.id().to_string(),
                SwapperProvider::Jupiter.id().to_string(),
            ])
        );
        let pancake_error = result.errors.iter().find(|e| e.provider.as_deref() == Some(SwapperProvider::PancakeswapV3.id())).unwrap();
        assert!(pancake_error.error.contains("1264000"));
    }

    #[test]
    fn test_sort_quotes_by_output_amount_desc() {
        let mut quotes = [
            Quote::mock_with_provider(SwapperProvider::UniswapV3, "101"),
            Quote::mock_with_provider(SwapperProvider::UniswapV4, "100"),
            Quote::mock_with_provider(SwapperProvider::PancakeswapV3, "102"),
        ];

        GemSwapper::sort_quotes_by_output_amount(&mut quotes);

        assert_eq!(quotes[0].to_value, "102");
        assert_eq!(quotes[1].to_value, "101");
        assert_eq!(quotes[2].to_value, "100");
    }
}
