use super::{
    asset::{supported_assets as mayan_supported_assets, token_id_for_asset},
    client::MayanClient,
    constants::{MAYAN_DEPOSIT_CONTRACTS, MAYAN_SEND_CONTRACTS},
    mapper::map_swap_result,
    model::{MayanChain, MayanQuote, QuoteParams, SwiftVersion},
    tx_builder::{fast_mctp, mctp, mono_chain, swift},
    wormhole_chain,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperQuoteData,
    config::get_swap_proxy_url,
    cross_chain::VaultAddresses,
    fees::{default_referral_address, default_referral_fees, quote_value_after_reserve, quote_value_after_reserve_by_chain},
};
use async_trait::async_trait;
use gem_client::Client;
use primitives::{Chain, ChainType};
use std::{collections::BTreeSet, fmt::Debug, sync::Arc};

const SOLANA_NATIVE_SWAP_RESERVE: &str = "5000000";

#[derive(Debug)]
pub struct Mayan<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    price_client: MayanClient<C>,
    explorer_client: MayanClient<C>,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl Mayan<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::with_clients(
            MayanClient::new(RpcClient::new(get_swap_proxy_url("mayan/price/v3"), rpc_provider.clone())),
            MayanClient::new(RpcClient::new(get_swap_proxy_url("mayan/explorer/v3"), rpc_provider.clone())),
            rpc_provider,
        )
    }
}

impl<C> Mayan<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn with_clients(price_client: MayanClient<C>, explorer_client: MayanClient<C>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Mayan),
            price_client,
            explorer_client,
            rpc_provider,
        }
    }

    fn supported_source_chain(chain: Chain) -> bool {
        match chain.chain_type() {
            ChainType::Ethereum | ChainType::Solana | ChainType::Sui => true,
            ChainType::Bitcoin
            | ChainType::Cosmos
            | ChainType::Ton
            | ChainType::Tron
            | ChainType::Aptos
            | ChainType::Xrp
            | ChainType::Near
            | ChainType::Stellar
            | ChainType::Algorand
            | ChainType::Polkadot
            | ChainType::Cardano
            | ChainType::HyperCore => false,
        }
    }

    fn supports_chain_pair(&self, from_chain: Chain, to_chain: Chain) -> bool {
        let supported_assets = mayan_supported_assets();
        Self::supported_source_chain(from_chain)
            && supported_assets.iter().any(|asset| asset.get_chain() == from_chain)
            && supported_assets.iter().any(|asset| asset.get_chain() == to_chain)
    }
}

#[async_trait]
impl<C> Swapper for Mayan<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        mayan_supported_assets()
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        if !self.supports_chain_pair(request.from_asset.chain(), request.to_asset.chain()) {
            return Err(SwapperError::NotSupportedChain);
        }

        let from_value = quote_value_after_mayan_reserve(request)?;
        let from_asset = request.from_asset.asset_id();
        let to_asset = request.to_asset.asset_id();
        let referral_fees = default_referral_fees();
        let routes = self
            .price_client
            .get_quotes(
                QuoteParams {
                    amount_in64: from_value.clone(),
                    from_token: token_id_for_asset(&from_asset),
                    from_chain: wormhole_chain::name_for_chain(from_asset.chain)?.to_string(),
                    to_token: token_id_for_asset(&to_asset),
                    to_chain: wormhole_chain::name_for_chain(to_asset.chain)?.to_string(),
                    referrer: default_referral_address(Chain::Solana),
                    referrer_bps: referral_fees.bps_for_chain(from_asset.chain),
                },
                request.from_asset.decimals,
            )
            .await?;
        let route = Self::select_route(&routes, from_asset.chain, to_asset.chain).ok_or(SwapperError::NoQuoteAvailable)?;
        let to_value = route.common().expected_output_value(request.to_asset.decimals)?;

        Ok(Quote {
            from_value,
            to_value,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: from_asset,
                    output: to_asset,
                    route_data: serde_json::to_string(route)?,
                }],
                slippage_bps: route.common().slippage_bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(route.common().eta_seconds),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route: MayanQuote = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        match (quote.request.from_asset.chain().chain_type(), &route) {
            (ChainType::Ethereum, MayanQuote::Swift(route)) => swift::evm::build_quote_data(&self.price_client, quote, route, self.rpc_provider.clone()).await,
            (ChainType::Ethereum, MayanQuote::Mctp(route)) => mctp::evm::build_quote_data(&self.price_client, quote, route, self.rpc_provider.clone()).await,
            (ChainType::Ethereum, MayanQuote::FastMctp(route)) => fast_mctp::evm::build_quote_data(&self.price_client, quote, route, self.rpc_provider.clone()).await,
            (ChainType::Ethereum, MayanQuote::MonoChain(route)) => mono_chain::evm::build_quote_data(quote, route, self.rpc_provider.clone()).await,
            (ChainType::Solana, MayanQuote::Swift(route)) => swift::solana::build_quote_data(&self.price_client, quote, route, self.rpc_provider.clone()).await,
            (ChainType::Solana, MayanQuote::Mctp(route)) => mctp::solana::build_quote_data(&self.price_client, quote, route, self.rpc_provider.clone()).await,
            (ChainType::Solana, MayanQuote::FastMctp(route)) => fast_mctp::solana::build_quote_data(&self.price_client, quote, route, self.rpc_provider.clone()).await,
            (ChainType::Sui, MayanQuote::Mctp(route)) => mctp::sui::build_quote_data(&self.price_client, quote, route, self.rpc_provider.clone()).await,
            (ChainType::Solana, MayanQuote::MonoChain(_)) | (ChainType::Sui, MayanQuote::Swift(_) | MayanQuote::FastMctp(_) | MayanQuote::MonoChain(_)) => {
                Err(SwapperError::InvalidRoute)
            }
            (
                ChainType::Bitcoin
                | ChainType::Cosmos
                | ChainType::Ton
                | ChainType::Tron
                | ChainType::Aptos
                | ChainType::Xrp
                | ChainType::Near
                | ChainType::Stellar
                | ChainType::Algorand
                | ChainType::Polkadot
                | ChainType::Cardano
                | ChainType::HyperCore,
                _,
            ) => Err(SwapperError::NotSupportedChain),
        }
    }

    async fn get_swap_result(&self, _chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let result = self.explorer_client.get_transaction_status(transaction_hash).await?;
        Ok(map_swap_result(&result))
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        let api_addresses = MayanChain::unique_addresses(self.price_client.get_chains().await?);
        let deposit: BTreeSet<String> = MAYAN_DEPOSIT_CONTRACTS.iter().map(|s| s.to_string()).chain(api_addresses.iter().cloned()).collect();
        let send: BTreeSet<String> = MAYAN_SEND_CONTRACTS.iter().map(|s| s.to_string()).chain(api_addresses).collect();

        Ok(VaultAddresses {
            deposit: deposit.into_iter().collect(),
            send: send.into_iter().collect(),
        })
    }
}

fn quote_value_after_mayan_reserve(request: &QuoteRequest) -> Result<String, SwapperError> {
    if request.options.use_max_amount && request.from_asset.chain() == Chain::Solana && request.from_asset.is_native() {
        return quote_value_after_reserve(request, SOLANA_NATIVE_SWAP_RESERVE);
    }
    quote_value_after_reserve_by_chain(request)
}

impl<C> Mayan<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn select_route(routes: &[MayanQuote], source_chain: Chain, destination_chain: Chain) -> Option<&MayanQuote> {
        if source_chain == Chain::Hyperliquid && destination_chain == Chain::HyperCore {
            return routes
                .iter()
                .find(|route| route.as_mono_chain().is_some())
                .or_else(|| {
                    routes
                        .iter()
                        .find(|route| route.as_swift().is_some_and(|swift| swift.swift_version == Some(SwiftVersion::V2)))
                })
                .or_else(|| routes.iter().find(|route| route.as_mctp().is_some()));
        }

        match source_chain.chain_type() {
            ChainType::Sui => routes.iter().find(|route| route.as_mctp().is_some()),
            ChainType::Ethereum => routes
                .iter()
                .find(|route| route.as_swift().is_some_and(|swift| swift.swift_version == Some(SwiftVersion::V2)))
                .or_else(|| routes.iter().find(|route| route.as_mono_chain().is_some()))
                .or_else(|| routes.iter().find(|route| route.as_fast_mctp().is_some()))
                .or_else(|| routes.iter().find(|route| route.as_mctp().is_some())),
            ChainType::Solana => routes
                .iter()
                .find(|route| route.as_swift().is_some_and(|swift| swift.swift_version == Some(SwiftVersion::V2)))
                .or_else(|| {
                    if destination_chain == Chain::Sui {
                        None
                    } else {
                        routes.iter().find(|route| route.as_fast_mctp().is_some())
                    }
                })
                .or_else(|| routes.iter().find(|route| route.as_mctp().is_some())),
            _ => routes
                .iter()
                .find(|route| route.as_swift().is_some_and(|swift| swift.swift_version == Some(SwiftVersion::V2)))
                .or_else(|| routes.iter().find(|route| route.as_mctp().is_some())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mayan::model::{MayanFastMctpQuote, MayanMctpQuote, MayanMonoChainQuote};
    use crate::models::Options;
    use crate::{SwapperQuoteAsset, alien::mock::ProviderMock};
    use gem_client::testkit::MockClient;
    use primitives::{
        AssetId,
        asset_constants::{ARBITRUM_USDC_ASSET_ID, HYPERCORE_SPOT_USDC_ASSET_ID, SOLANA_USDC_TOKEN_ID},
    };
    use std::collections::BTreeSet;

    #[tokio::test]
    async fn test_get_vault_addresses() {
        let price_client = MockClient::new().with_get(|path| {
            assert_eq!(path, "/chains");
            Ok(br#"[
                    {"mayanAddress":"0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
                    {"mayanAddress":"0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
                    {"mayanAddress":""}
                ]"#
            .to_vec())
        });
        let provider = Mayan::with_clients(
            MayanClient::new(price_client),
            MayanClient::new(MockClient::new()),
            Arc::new(ProviderMock::new("{}".to_string())),
        );

        let addresses = provider.get_vault_addresses(None).await.unwrap();
        let api_address = gem_evm::ethereum_address_checksum("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
        let expected_deposit = MAYAN_DEPOSIT_CONTRACTS
            .iter()
            .map(|address| address.to_string())
            .chain([api_address.clone()])
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let expected_send = MAYAN_SEND_CONTRACTS
            .iter()
            .map(|address| address.to_string())
            .chain([api_address])
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        assert_eq!(addresses.deposit, expected_deposit);
        assert_eq!(addresses.send, expected_send);
    }

    #[tokio::test]
    async fn test_get_quote_rescales_mayan_base_units_to_destination_asset_decimals() {
        let price_client = MockClient::new().with_get(|path| {
            assert!(path.starts_with("/quote?"));
            Ok(include_bytes!("test/quote_response_swift_hypercore.json").to_vec())
        });
        let provider = Mayan::with_clients(
            MayanClient::new(price_client),
            MayanClient::new(MockClient::new()),
            Arc::new(ProviderMock::new("{}".to_string())),
        );
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset {
                id: ARBITRUM_USDC_ASSET_ID.to_string(),
                symbol: "USDC".to_string(),
                decimals: 6,
            },
            to_asset: SwapperQuoteAsset {
                id: HYPERCORE_SPOT_USDC_ASSET_ID.to_string(),
                symbol: "USDC".to_string(),
                decimals: 8,
            },
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "7000000".to_string(),
            options: Options::new_with_slippage(5.into()),
        };

        let quote = provider.get_quote(&request).await.unwrap();

        assert_eq!(quote.to_value, "602333700");
    }

    #[test]
    fn test_select_route_falls_back_to_mctp_for_non_sui_source() {
        let routes = vec![MayanQuote::Mctp(Box::new(MayanMctpQuote::mock()))];

        assert!(Mayan::<MockClient>::select_route(&routes, Chain::Ethereum, Chain::Sui).unwrap().as_mctp().is_some());
        assert!(Mayan::<MockClient>::select_route(&routes, Chain::Solana, Chain::Sui).unwrap().as_mctp().is_some());
    }

    #[test]
    fn test_select_route_prefers_fast_mctp_before_mctp_when_supported() {
        let routes = vec![
            MayanQuote::Mctp(Box::new(MayanMctpQuote::mock())),
            MayanQuote::FastMctp(Box::new(MayanFastMctpQuote::mock())),
        ];

        assert!(Mayan::<MockClient>::select_route(&routes, Chain::Ethereum, Chain::Base).unwrap().as_fast_mctp().is_some());
        assert!(Mayan::<MockClient>::select_route(&routes, Chain::Solana, Chain::Base).unwrap().as_fast_mctp().is_some());
    }

    #[test]
    fn test_select_route_keeps_mctp_for_solana_to_sui_fast_mctp_gap() {
        let routes = vec![
            MayanQuote::FastMctp(Box::new(MayanFastMctpQuote::mock())),
            MayanQuote::Mctp(Box::new(MayanMctpQuote::mock())),
        ];

        assert!(Mayan::<MockClient>::select_route(&routes, Chain::Solana, Chain::Sui).unwrap().as_mctp().is_some());
    }

    #[test]
    fn test_select_route_prefers_mono_chain_for_hyperevm_to_hypercore() {
        let routes = vec![
            MayanQuote::Mctp(Box::new(MayanMctpQuote::mock())),
            MayanQuote::MonoChain(Box::new(MayanMonoChainQuote::default())),
        ];

        assert!(
            Mayan::<MockClient>::select_route(&routes, Chain::Hyperliquid, Chain::HyperCore)
                .unwrap()
                .as_mono_chain()
                .is_some()
        );
    }

    #[test]
    fn test_supports_chain_pair_allows_hyperevm_to_hypercore_only() {
        let provider = Mayan::with_clients(
            MayanClient::new(MockClient::new()),
            MayanClient::new(MockClient::new()),
            Arc::new(ProviderMock::new("{}".to_string())),
        );

        assert!(provider.supports_chain_pair(Chain::Hyperliquid, Chain::HyperCore));
        assert!(!provider.supports_chain_pair(Chain::HyperCore, Chain::Hyperliquid));
    }

    #[test]
    fn test_quote_value_after_mayan_reserve_uses_larger_solana_native_reserve() {
        let mut request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            wallet_address: "address".to_string(),
            destination_address: "address".to_string(),
            value: "105814789".to_string(),
            options: Options {
                use_max_amount: true,
                ..Default::default()
            },
        };

        assert_eq!(quote_value_after_mayan_reserve(&request).unwrap(), "100814789");

        request.from_asset = SwapperQuoteAsset::from(AssetId::from_token(Chain::Solana, SOLANA_USDC_TOKEN_ID));
        assert_eq!(quote_value_after_mayan_reserve(&request).unwrap(), "105814789");
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{FetchQuoteData, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, mayan::constants::MAYAN_FORWARDER, models::Options};
    use primitives::{
        AssetId,
        asset_constants::{BASE_USDC_ASSET_ID, HYPERCORE_SPOT_USDC_ASSET_ID, HYPEREVM_USDC_ASSET_ID, POLYGON_USDT_ASSET_ID, SOLANA_USDC_ASSET_ID, SUI_USDC_ASSET_ID},
        swap::SwapStatus,
    };
    use std::{future::Future, time::Instant};

    async fn timed<T, E>(label: &str, future: impl Future<Output = Result<T, E>>) -> Result<T, E> {
        let started_at = Instant::now();
        let result = future.await;
        println!("{label}: {:?}", started_at.elapsed());
        result
    }

    fn mayan_route(quote: &Quote) -> Result<MayanQuote, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        serde_json::from_str(&route.route_data).map_err(SwapperError::from)
    }

    #[tokio::test]
    async fn test_mayan_provider_get_swift_evm_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default().set_debug(false));
        let provider = Mayan::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            value: "50000000000000000".to_string(),
            options: Options::new_with_slippage(200.into()),
        };

        let quote = timed("mayan swift evm quote", provider.get_quote(&request)).await?;
        let quote_data = timed("mayan swift evm quote data", provider.get_quote_data(&quote, FetchQuoteData::None)).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);
        let MayanQuote::Swift(route) = mayan_route(&quote)? else {
            return Err(SwapperError::InvalidRoute);
        };
        assert_eq!(route.from_chain, wormhole_chain::WormholeChain::Ethereum.name());
        assert_eq!(route.to_chain, wormhole_chain::WormholeChain::Solana.name());
        assert!(!quote_data.to.is_empty());
        assert!(!quote_data.data.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_mayan_provider_get_swift_solana_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default().set_debug(false));
        let provider = Mayan::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "5000000".to_string(),
            options: Options::new_with_slippage(200.into()),
        };

        let quote = timed("mayan swift solana quote", provider.get_quote(&request)).await?;
        let quote_data = timed("mayan swift solana quote data", provider.get_quote_data(&quote, FetchQuoteData::None)).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);
        let MayanQuote::Swift(route) = mayan_route(&quote)? else {
            return Err(SwapperError::InvalidRoute);
        };
        assert_eq!(route.from_chain, wormhole_chain::WormholeChain::Solana.name());
        assert_eq!(route.to_chain, wormhole_chain::WormholeChain::Base.name());
        assert!(quote_data.to.is_empty());
        assert_eq!(quote_data.value, "0");
        assert!(!quote_data.data.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_mayan_provider_get_mctp_sui_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default().set_debug(false));
        let provider = Mayan::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "1000000000".to_string(),
            options: Options::new_with_slippage(200.into()),
        };

        let quote = timed("mayan mctp sui quote", provider.get_quote(&request)).await?;
        let quote_data = timed("mayan mctp sui quote data", provider.get_quote_data(&quote, FetchQuoteData::None)).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);
        let MayanQuote::Mctp(route) = mayan_route(&quote)? else {
            return Err(SwapperError::InvalidRoute);
        };
        assert_eq!(route.from_chain, wormhole_chain::WormholeChain::Sui.name());
        assert_eq!(route.to_chain, wormhole_chain::WormholeChain::Base.name());
        assert!(quote_data.to.is_empty());
        assert_eq!(quote_data.value, "0");
        assert!(!quote_data.data.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_mayan_provider_get_mctp_solana_to_sui_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default().set_debug(false));
        let provider = Mayan::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(SUI_USDC_ASSET_ID.clone()),
            wallet_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            destination_address: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".to_string(),
            value: "1000000".to_string(),
            options: Options::new_with_slippage(200.into()),
        };

        let quote = timed("mayan mctp solana to sui quote", provider.get_quote(&request)).await?;
        let quote_data = timed("mayan mctp solana to sui quote data", provider.get_quote_data(&quote, FetchQuoteData::None)).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        let MayanQuote::Mctp(route) = mayan_route(&quote)? else {
            return Err(SwapperError::InvalidRoute);
        };
        assert_eq!(route.from_chain, wormhole_chain::WormholeChain::Solana.name());
        assert_eq!(route.to_chain, wormhole_chain::WormholeChain::Sui.name());
        assert!(quote_data.to.is_empty());
        assert_eq!(quote_data.value, "0");
        assert!(!quote_data.data.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_mayan_provider_get_mctp_evm_to_sui_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default().set_debug(false));
        let provider = Mayan::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
            to_asset: SwapperQuoteAsset::from(SUI_USDC_ASSET_ID.clone()),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".to_string(),
            value: "100000000000000000".to_string(),
            options: Options::new_with_slippage(200.into()),
        };

        let quote = timed("mayan mctp evm to sui quote", provider.get_quote(&request)).await?;
        let quote_data = timed("mayan mctp evm to sui quote data", provider.get_quote_data(&quote, FetchQuoteData::None)).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        let MayanQuote::Mctp(route) = mayan_route(&quote)? else {
            return Err(SwapperError::InvalidRoute);
        };
        assert_eq!(route.from_chain, wormhole_chain::WormholeChain::Ethereum.name());
        assert_eq!(route.to_chain, wormhole_chain::WormholeChain::Sui.name());
        assert_eq!(quote_data.to, MAYAN_FORWARDER);
        assert!(!quote_data.data.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_mayan_provider_get_mono_chain_hyperevm_to_hypercore_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default().set_debug(false));
        let provider = Mayan::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(HYPEREVM_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(HYPERCORE_SPOT_USDC_ASSET_ID.clone()),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "1000000".to_string(),
            options: Options::new_with_slippage(200.into()),
        };

        let quote = timed("mayan mono-chain hyperevm to hypercore quote", provider.get_quote(&request)).await?;
        let quote_data = timed("mayan mono-chain hyperevm to hypercore quote data", provider.get_quote_data(&quote, FetchQuoteData::None)).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        let MayanQuote::MonoChain(route) = mayan_route(&quote)? else {
            return Err(SwapperError::InvalidRoute);
        };
        assert_eq!(route.from_chain, wormhole_chain::WormholeChain::Hyperevm.name());
        assert_eq!(route.to_chain, wormhole_chain::WormholeChain::Hypercore.name());
        assert_eq!(quote_data.to, MAYAN_FORWARDER);
        assert!(!quote_data.data.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_mayan_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rpc_provider = Arc::new(NativeProvider::default().set_debug(false));
        let provider = Mayan::new(rpc_provider);
        let hash = "0xfb2464f06d38f39a274b2a5e3414dbed43ad405a06295aaeaded8865efc7d4f4";
        let result = provider.get_swap_result(Chain::Ethereum, hash).await?;

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, POLYGON_USDT_ASSET_ID.clone());
        assert_eq!(metadata.from_value, "35245466");
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::Base, "0xEF5997c2cf2f6c138196f8A6203afc335206b3c1"));
        assert_eq!(metadata.to_value, "398724622644505839482");
        assert_eq!(metadata.provider, Some("mayan".to_string()));
        Ok(())
    }
}
