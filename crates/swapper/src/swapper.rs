use crate::{
    AssetList, FetchQuoteData, Permit2ApprovalData, ProviderType, Quote, QuoteRequest, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperProviderMode, SwapperQuoteData, across, alien::RpcProvider, chainflip, config::DEFAULT_STABLE_SWAP_REFERRAL_BPS, hyperliquid, jupiter, near_intents,
    pancakeswap_aptos, proxy::provider_factory, thorchain, uniswap,
};
use num_traits::ToPrimitive;
use primitives::{AssetId, Chain, EVMChain};
use std::{borrow::Cow, collections::HashSet, fmt::Debug, sync::Arc};

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

    fn transform_request<'a>(request: &'a QuoteRequest) -> Cow<'a, QuoteRequest> {
        if !Self::is_stable_swap(request) || request.options.fee.is_none() {
            return Cow::Borrowed(request);
        }

        let mut updated_request = request.clone();
        if let Some(fees) = updated_request.options.fee.as_mut() {
            fees.update_all_bps(DEFAULT_STABLE_SWAP_REFERRAL_BPS);
        }

        Cow::Owned(updated_request)
    }

    fn is_stable_swap(request: &QuoteRequest) -> bool {
        let from_symbol = request.from_asset.symbol.to_ascii_uppercase();
        let to_symbol = request.to_asset.symbol.to_ascii_uppercase();

        from_symbol.contains("USD") && to_symbol.contains("USD")
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
            Box::new(across::Across::new(rpc_provider.clone())),
            Box::new(hyperliquid::Hyperliquid::new(rpc_provider.clone())),
            uniswap::default::boxed_oku(rpc_provider.clone()),
            uniswap::default::boxed_wagmi(rpc_provider.clone()),
            Box::new(pancakeswap_aptos::PancakeSwapAptos::new(rpc_provider.clone())),
            Box::new(provider_factory::new_stonfi_v2(rpc_provider.clone())),
            Box::new(provider_factory::new_mayan(rpc_provider.clone())),
            Box::new(near_intents::NearIntents::new(rpc_provider.clone())),
            Box::new(chainflip::ChainflipProvider::new(rpc_provider.clone())),
            Box::new(provider_factory::new_cetus_aggregator(rpc_provider.clone())),
            Box::new(provider_factory::new_relay(rpc_provider.clone())),
            Box::new(provider_factory::new_orca(rpc_provider.clone())),
            uniswap::default::boxed_aerodrome(rpc_provider.clone()),
        ];

        Self { rpc_provider, swappers }
    }

    pub fn supported_chains(&self) -> Vec<Chain> {
        self.swappers
            .iter()
            .flat_map(|x| x.supported_chains())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
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

    pub async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Vec<Quote>, SwapperError> {
        if request.from_asset.id == request.to_asset.id {
            return Err(SwapperError::NotSupportedPair);
        }
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        let preferred_providers = &request.options.preferred_providers;
        let providers = self
            .swappers
            .iter()
            .filter(|x| Self::filter_by_provider_mode(&x.provider().mode, from_chain, to_chain))
            .filter(|x| Self::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .filter(|x| Self::filter_by_preferred_providers(preferred_providers, &x.provider().id))
            .collect::<Vec<_>>();

        if providers.is_empty() {
            return Err(SwapperError::NoAvailableProvider);
        }

        let request_for_quote = Self::transform_request(request);
        let quotes_futures = providers.into_iter().map(|x| x.fetch_quote(request_for_quote.as_ref()));

        let quotes = futures::future::join_all(quotes_futures.into_iter().map(|fut| async {
            match &fut.await {
                Ok(quote) => Some(quote.clone()),
                Err(_err) => {
                    tracing::debug!("fetch_quote error: {:?}", _err);
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

    pub async fn fetch_quote_by_provider(&self, provider: SwapperProvider, request: QuoteRequest) -> Result<Quote, SwapperError> {
        let provider = self.get_swapper_by_provider(&provider).ok_or(SwapperError::NoAvailableProvider)?;
        let request_for_quote = Self::transform_request(&request);
        provider.fetch_quote(request_for_quote.as_ref()).await
    }

    pub async fn fetch_permit2_for_quote(&self, quote: &Quote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let provider = self.get_swapper_by_provider(&quote.data.provider.id).ok_or(SwapperError::NoAvailableProvider)?;
        provider.fetch_permit2_for_quote(quote).await
    }

    pub async fn fetch_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let provider = self.get_swapper_by_provider(&quote.data.provider.id).ok_or(SwapperError::NoAvailableProvider)?;
        let mut quote_data = provider.fetch_quote_data(quote, data).await?;
        if let Some(gas_limit) = quote_data.gas_limit.take() {
            quote_data.gas_limit = Some(Self::apply_gas_limit_multiplier(&quote.request.from_asset.chain(), gas_limit));
        }
        Ok(quote_data)
    }

    pub async fn get_swap_result(&self, chain: Chain, swap_provider: SwapperProvider, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let provider = self.get_swapper_by_provider(&swap_provider).ok_or(SwapperError::NoAvailableProvider)?;
        provider.get_swap_result(chain, transaction_hash).await
    }
}

#[cfg(all(test, feature = "reqwest_provider"))]
mod tests {

    use super::*;
    use crate::{
        Options, SwapperMode, SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode,
        alien::reqwest_provider::NativeProvider,
        config::{DEFAULT_STABLE_SWAP_REFERRAL_BPS, DEFAULT_SWAP_FEE_BPS, ReferralFees},
        uniswap::default::{new_pancakeswap, new_uniswap_v3},
    };
    use primitives::asset_constants::USDT_ETH_ASSET_ID;
    use std::{borrow::Cow, collections::BTreeSet, sync::Arc, vec};

    fn build_request(from_symbol: &str, to_symbol: &str, fee: Option<ReferralFees>) -> QuoteRequest {
        QuoteRequest {
            from_asset: SwapperQuoteAsset {
                id: format!("{}_asset", from_symbol),
                symbol: from_symbol.to_string(),
                decimals: 6,
            },
            to_asset: SwapperQuoteAsset {
                id: format!("{}_asset", to_symbol),
                symbol: to_symbol.to_string(),
                decimals: 6,
            },
            wallet_address: "0xwallet".into(),
            destination_address: "0xwallet".into(),
            value: "1000000".into(),
            mode: SwapperMode::ExactIn,
            options: Options {
                slippage: SwapperSlippage {
                    bps: 100,
                    mode: SwapperSlippageMode::Exact,
                },
                fee,
                preferred_providers: vec![],
                use_max_amount: false,
            },
        }
    }

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
        let asset_id_usdt: AssetId = USDT_ETH_ASSET_ID.into();
        let supported_assets_all = vec![SwapperChainAsset::All(Chain::Ethereum)];
        assert!(GemSwapper::filter_supported_assets(supported_assets_all, asset_id.clone()));

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

    #[test]
    fn test_is_stable_swap_detection() {
        let stable_request = build_request("USDC", "USDT", None);
        assert!(GemSwapper::is_stable_swap(&stable_request));

        let non_stable_request = build_request("ETH", "USDC", None);
        assert!(!GemSwapper::is_stable_swap(&non_stable_request));
    }

    #[test]
    fn test_stable_swap_adjusts_fees() {
        use crate::config::get_swap_config;

        let request = build_request("USDC", "USDT", Some(get_swap_config().referral_fee));

        let adjusted_request = match GemSwapper::transform_request(&request) {
            Cow::Owned(req) => req,
            Cow::Borrowed(_) => panic!("stable swap should adjust request"),
        };
        let adjusted_fees = adjusted_request.options.fee.unwrap();

        assert_eq!(adjusted_fees.evm.bps, DEFAULT_STABLE_SWAP_REFERRAL_BPS);
        assert_eq!(adjusted_fees.evm_bridge.bps, DEFAULT_STABLE_SWAP_REFERRAL_BPS);
        assert_eq!(adjusted_fees.solana.bps, DEFAULT_STABLE_SWAP_REFERRAL_BPS);
        assert_eq!(adjusted_fees.thorchain.bps, DEFAULT_STABLE_SWAP_REFERRAL_BPS);
        assert_eq!(adjusted_fees.sui.bps, DEFAULT_STABLE_SWAP_REFERRAL_BPS);
        assert_eq!(adjusted_fees.ton.bps, DEFAULT_STABLE_SWAP_REFERRAL_BPS);
        assert_eq!(adjusted_fees.tron.bps, DEFAULT_STABLE_SWAP_REFERRAL_BPS);

        let original_fees = request.options.fee.as_ref().unwrap();
        assert_eq!(original_fees.evm.bps, DEFAULT_SWAP_FEE_BPS);
    }

    #[test]
    fn test_transform_request_skips_when_not_applicable() {
        let non_stable_request = build_request("ETH", "USDC", None);
        assert!(matches!(GemSwapper::transform_request(&non_stable_request), Cow::Borrowed(_)));

        let stable_without_fees = build_request("USDC", "USDT", None);
        assert!(matches!(GemSwapper::transform_request(&stable_without_fees), Cow::Borrowed(_)));
    }
}
