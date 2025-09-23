use crate::swapper::{SwapperProvider, SwapperProviderType, asset::*, models::SwapperChainAsset};
use primitives::Chain;

use super::provider::{PROVIDER_API_URL, ProxyProvider};

impl ProxyProvider {
    pub fn new_stonfi_v2() -> ProxyProvider {
        ProxyProvider {
            provider: SwapperProviderType::new(SwapperProvider::StonfiV2),
            url: format!("{}/{}", PROVIDER_API_URL, "stonfi_v2"),
            assets: vec![SwapperChainAsset::All(Chain::Ton)],
        }
    }

    pub fn new_symbiosis() -> ProxyProvider {
        ProxyProvider {
            provider: SwapperProviderType::new(SwapperProvider::Symbiosis),
            url: format!("{}/{}", PROVIDER_API_URL, "symbiosis"),
            assets: vec![SwapperChainAsset::All(Chain::Tron)],
        }
    }

    pub fn new_cetus_aggregator() -> ProxyProvider {
        ProxyProvider {
            provider: SwapperProviderType::new(SwapperProvider::CetusAggregator),
            url: format!("{}/{}", PROVIDER_API_URL, "cetus"),
            assets: vec![SwapperChainAsset::All(Chain::Sui)],
        }
    }

    pub fn new_mayan() -> ProxyProvider {
        ProxyProvider {
            provider: SwapperProviderType::new(SwapperProvider::Mayan),
            url: format!("{}/{}", PROVIDER_API_URL, "mayan"),
            assets: vec![
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
            ],
        }
    }

    pub fn new_relay() -> ProxyProvider {
        ProxyProvider {
            provider: SwapperProviderType::new(SwapperProvider::Relay),
            url: format!("{}/{}", PROVIDER_API_URL, "relay"),
            assets: vec![
                SwapperChainAsset::All(Chain::Hyperliquid),
                SwapperChainAsset::All(Chain::Manta),
                SwapperChainAsset::All(Chain::Berachain),
            ],
        }
    }
}
