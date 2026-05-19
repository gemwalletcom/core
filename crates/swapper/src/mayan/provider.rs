use super::{
    client::MayanClient,
    constants::{HYPERCORE_SPOT_USDC_CONTRACT, MAYAN_DEPOSIT_CONTRACTS, MAYAN_SEND_CONTRACTS},
    mapper::map_swap_result,
    model::{MayanChain, MayanQuote, MayanQuoteCommon, QuoteParams, SwiftVersion},
    tx_builder::{mctp, swift},
    wormhole_chain,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperQuoteData,
    config::get_swap_proxy_url,
    cross_chain::VaultAddresses,
    fees::{ReferralFees, default_referral_address, default_referral_fees, quote_value_after_reserve_by_chain},
};
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::EVM_ZERO_ADDRESS;
use gem_solana::WSOL_TOKEN_ADDRESS;
use gem_sui::SUI_COIN_TYPE;
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{
    AssetId, Chain, ChainType,
    asset_constants::{
        ARBITRUM_USDC_ASSET_ID, ARBITRUM_USDT_ASSET_ID, AVALANCHE_USDC_ASSET_ID, AVALANCHE_USDT_ASSET_ID, BASE_CBBTC_ASSET_ID, BASE_USDC_ASSET_ID, BASE_USDS_ASSET_ID,
        BASE_WBTC_ASSET_ID, ETHEREUM_CBBTC_ASSET_ID, ETHEREUM_DAI_ASSET_ID, ETHEREUM_STETH_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDS_ASSET_ID, ETHEREUM_USDT_ASSET_ID,
        ETHEREUM_WBTC_ASSET_ID, ETHEREUM_WETH_ASSET_ID, HYPERCORE_SPOT_USDC_ASSET_ID, HYPERCORE_SPOT_USDC_TOKEN_ID, HYPEREVM_USDC_ASSET_ID, HYPEREVM_USDT_ASSET_ID,
        LINEA_USDC_E_ASSET_ID, LINEA_USDT_ASSET_ID, MONAD_USDC_ASSET_ID, MONAD_USDT_ASSET_ID, OPTIMISM_USDC_ASSET_ID, OPTIMISM_USDT_ASSET_ID, POLYGON_USDC_ASSET_ID,
        POLYGON_USDT_ASSET_ID, SMARTCHAIN_USDC_ASSET_ID, SMARTCHAIN_USDT_ASSET_ID, SMARTCHAIN_WBTC_ASSET_ID, SOLANA_CBBTC_ASSET_ID, SOLANA_JITO_SOL_ASSET_ID, SOLANA_USDC_ASSET_ID,
        SOLANA_USDS_ASSET_ID, SOLANA_USDT_ASSET_ID, SOLANA_WBTC_ASSET_ID, SUI_SBUSDT_ASSET_ID, SUI_USDC_ASSET_ID, SUI_WAL_ASSET_ID, UNICHAIN_DAI_ASSET_ID, UNICHAIN_USDC_ASSET_ID,
    },
};
use serde_json::Value;
use std::{collections::BTreeSet, fmt::Debug, str::FromStr, sync::Arc};

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

    fn referral_fees(request: &QuoteRequest) -> ReferralFees {
        request.options.fee.clone().unwrap_or_else(default_referral_fees)
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

    fn map_asset_to_token_id(asset_id: &AssetId) -> String {
        match (asset_id.chain, asset_id.token_id.as_deref()) {
            (Chain::Solana, None) => WSOL_TOKEN_ADDRESS.to_string(),
            (Chain::Sui, None) => SUI_COIN_TYPE.to_string(),
            (Chain::HyperCore, Some(HYPERCORE_SPOT_USDC_TOKEN_ID)) => HYPERCORE_SPOT_USDC_CONTRACT.to_string(),
            (_, None) => EVM_ZERO_ADDRESS.to_string(),
            (_, Some(token_id)) => token_id.to_string(),
        }
    }

    fn quote_output_value(route: &MayanQuoteCommon) -> Result<String, SwapperError> {
        if let Some(value) = &route.expected_amount_out_base_units {
            return BigUint::from_str(value).map(|amount| amount.to_string()).map_err(SwapperError::from);
        }
        let amount = match &route.expected_amount_out {
            Value::Number(number) => number.to_string(),
            Value::String(value) => value.clone(),
            _ => return Err(SwapperError::InvalidRoute),
        };
        BigNumberFormatter::value_from_amount(&amount, route.to_token.decimals).map_err(SwapperError::from)
    }

    fn referral_bps(request: &QuoteRequest, referral_fees: &ReferralFees) -> u32 {
        match request.from_asset.chain().chain_type() {
            ChainType::Ethereum => referral_fees.evm.bps,
            ChainType::Solana => referral_fees.solana.bps,
            ChainType::Sui => referral_fees.sui.bps,
            _ => referral_fees.solana.bps,
        }
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
        vec![
            SwapperChainAsset::Assets(
                Chain::Ethereum,
                vec![
                    ETHEREUM_USDT_ASSET_ID.clone(),
                    ETHEREUM_USDC_ASSET_ID.clone(),
                    ETHEREUM_DAI_ASSET_ID.clone(),
                    ETHEREUM_USDS_ASSET_ID.clone(),
                    ETHEREUM_WBTC_ASSET_ID.clone(),
                    ETHEREUM_WETH_ASSET_ID.clone(),
                    ETHEREUM_STETH_ASSET_ID.clone(),
                    ETHEREUM_CBBTC_ASSET_ID.clone(),
                ],
            ),
            SwapperChainAsset::Assets(
                Chain::Solana,
                vec![
                    SOLANA_USDC_ASSET_ID.clone(),
                    SOLANA_USDT_ASSET_ID.clone(),
                    SOLANA_USDS_ASSET_ID.clone(),
                    SOLANA_CBBTC_ASSET_ID.clone(),
                    SOLANA_WBTC_ASSET_ID.clone(),
                    SOLANA_JITO_SOL_ASSET_ID.clone(),
                ],
            ),
            SwapperChainAsset::Assets(Chain::Sui, vec![SUI_USDC_ASSET_ID.clone(), SUI_SBUSDT_ASSET_ID.clone(), SUI_WAL_ASSET_ID.clone()]),
            SwapperChainAsset::Assets(
                Chain::SmartChain,
                vec![SMARTCHAIN_USDT_ASSET_ID.clone(), SMARTCHAIN_USDC_ASSET_ID.clone(), SMARTCHAIN_WBTC_ASSET_ID.clone()],
            ),
            SwapperChainAsset::Assets(
                Chain::Base,
                vec![
                    BASE_USDC_ASSET_ID.clone(),
                    BASE_CBBTC_ASSET_ID.clone(),
                    BASE_WBTC_ASSET_ID.clone(),
                    BASE_USDS_ASSET_ID.clone(),
                ],
            ),
            SwapperChainAsset::Assets(Chain::Polygon, vec![POLYGON_USDC_ASSET_ID.clone(), POLYGON_USDT_ASSET_ID.clone()]),
            SwapperChainAsset::Assets(Chain::AvalancheC, vec![AVALANCHE_USDT_ASSET_ID.clone(), AVALANCHE_USDC_ASSET_ID.clone()]),
            SwapperChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDC_ASSET_ID.clone(), ARBITRUM_USDT_ASSET_ID.clone()]),
            SwapperChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_USDC_ASSET_ID.clone(), OPTIMISM_USDT_ASSET_ID.clone()]),
            SwapperChainAsset::Assets(Chain::Linea, vec![LINEA_USDC_E_ASSET_ID.clone(), LINEA_USDT_ASSET_ID.clone()]),
            SwapperChainAsset::Assets(Chain::Unichain, vec![UNICHAIN_USDC_ASSET_ID.clone(), UNICHAIN_DAI_ASSET_ID.clone()]),
            SwapperChainAsset::Assets(Chain::Monad, vec![Chain::Monad.as_asset_id(), MONAD_USDC_ASSET_ID.clone(), MONAD_USDT_ASSET_ID.clone()]),
            SwapperChainAsset::Assets(Chain::Hyperliquid, vec![HYPEREVM_USDT_ASSET_ID.clone(), HYPEREVM_USDC_ASSET_ID.clone()]),
            SwapperChainAsset::Assets(Chain::HyperCore, vec![HYPERCORE_SPOT_USDC_ASSET_ID.clone()]),
        ]
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        if !Self::supported_source_chain(request.from_asset.chain()) {
            return Err(SwapperError::NotSupportedChain);
        }

        let from_value = quote_value_after_reserve_by_chain(request)?;
        let from_asset = request.from_asset.asset_id();
        let to_asset = request.to_asset.asset_id();
        let referral_fees = Self::referral_fees(request);
        let routes = self
            .price_client
            .fetch_quotes(
                QuoteParams {
                    amount_in64: from_value.clone(),
                    from_token: Self::map_asset_to_token_id(&from_asset),
                    from_chain: wormhole_chain::name_for_chain(from_asset.chain)?.to_string(),
                    to_token: Self::map_asset_to_token_id(&to_asset),
                    to_chain: wormhole_chain::name_for_chain(to_asset.chain)?.to_string(),
                    referrer: default_referral_address(Chain::Solana),
                    referrer_bps: Self::referral_bps(request, &referral_fees),
                },
                request.from_asset.decimals,
            )
            .await?;
        let route = Self::select_route(&routes, from_asset.chain.chain_type()).ok_or(SwapperError::NoQuoteAvailable)?;
        let to_value = Self::quote_output_value(route.common())?;

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
        match quote.request.from_asset.chain().chain_type() {
            ChainType::Ethereum => {
                let route = route.as_swift().ok_or(SwapperError::InvalidRoute)?;
                swift::evm::build_quote_data(&self.price_client, quote, route, self.rpc_provider.clone()).await
            }
            ChainType::Solana => {
                let route = route.as_swift().ok_or(SwapperError::InvalidRoute)?;
                swift::solana::build_quote_data(&self.price_client, quote, route, self.rpc_provider.clone()).await
            }
            ChainType::Sui => {
                let route = route.as_mctp().ok_or(SwapperError::InvalidRoute)?;
                mctp::sui::build_quote_data(&self.price_client, quote, route, self.rpc_provider.clone()).await
            }
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
            | ChainType::HyperCore => Err(SwapperError::NotSupportedChain),
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

impl<C> Mayan<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn select_route(routes: &[MayanQuote], source_chain_type: ChainType) -> Option<&MayanQuote> {
        match source_chain_type {
            ChainType::Sui => routes.iter().find(|route| route.as_mctp().is_some()),
            _ => routes
                .iter()
                .find(|route| route.as_swift().is_some_and(|swift| swift.swift_version == Some(SwiftVersion::V2))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::mock::ProviderMock;
    use crate::mayan::model::MayanToken;
    use gem_client::testkit::MockClient;
    use std::collections::BTreeSet;

    #[test]
    fn test_map_asset_to_token_id() {
        assert_eq!(Mayan::<MockClient>::map_asset_to_token_id(&AssetId::from_chain(Chain::Ethereum)), EVM_ZERO_ADDRESS);
        assert_eq!(Mayan::<MockClient>::map_asset_to_token_id(&AssetId::from_chain(Chain::Solana)), WSOL_TOKEN_ADDRESS);
        assert_eq!(Mayan::<MockClient>::map_asset_to_token_id(&AssetId::from_chain(Chain::Sui)), SUI_COIN_TYPE);
        assert_eq!(Mayan::<MockClient>::map_asset_to_token_id(&HYPERCORE_SPOT_USDC_ASSET_ID), HYPERCORE_SPOT_USDC_CONTRACT);
        assert_eq!(
            Mayan::<MockClient>::map_asset_to_token_id(&BASE_USDC_ASSET_ID),
            BASE_USDC_ASSET_ID.token_id.clone().unwrap()
        );
    }

    #[test]
    fn test_quote_output_value() {
        let mut route = MayanQuoteCommon {
            expected_amount_out_base_units: Some("1237897283".to_string()),
            ..Default::default()
        };
        assert_eq!(Mayan::<MockClient>::quote_output_value(&route).unwrap(), "1237897283");

        route.expected_amount_out_base_units = None;
        route.expected_amount_out = serde_json::json!(1.237897283);
        route.to_token = MayanToken {
            decimals: 9,
            ..Default::default()
        };
        assert_eq!(Mayan::<MockClient>::quote_output_value(&route).unwrap(), "1237897283");
    }

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
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{FetchQuoteData, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use primitives::{
        AssetId,
        asset_constants::{BASE_USDC_ASSET_ID, POLYGON_USDT_ASSET_ID, SOLANA_USDC_ASSET_ID},
        swap::SwapStatus,
    };

    fn mayan_route(quote: &Quote) -> Result<MayanQuote, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        serde_json::from_str(&route.route_data).map_err(SwapperError::from)
    }

    #[tokio::test]
    async fn test_mayan_provider_fetch_swift_evm_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Mayan::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            value: "50000000000000000".to_string(),
            options: Options::new_with_slippage(200.into()),
        };

        let quote = provider.get_quote(&request).await?;
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;

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
    async fn test_mayan_provider_fetch_swift_solana_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Mayan::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "5000000".to_string(),
            options: Options::new_with_slippage(200.into()),
        };

        let quote = provider.get_quote(&request).await?;
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;

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
    async fn test_mayan_provider_fetch_mctp_sui_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Mayan::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "1000000000".to_string(),
            options: Options::new_with_slippage(200.into()),
        };

        let quote = provider.get_quote(&request).await?;
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;

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
    async fn test_mayan_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Mayan::new(rpc_provider);
        let tx_hash = "0xfb2464f06d38f39a274b2a5e3414dbed43ad405a06295aaeaded8865efc7d4f4";
        let result = provider.get_swap_result(Chain::Ethereum, tx_hash).await?;

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
