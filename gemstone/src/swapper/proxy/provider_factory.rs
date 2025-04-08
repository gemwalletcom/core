use crate::swapper::{asset::*, models::SwapChainAsset, SwapProvider, SwapProviderType};
use primitives::Chain;

use super::provider::{ProxyProvider, PROVIDER_API_URL};

pub fn new_stonfi_v2() -> ProxyProvider {
    ProxyProvider {
        provider: SwapProviderType::new(SwapProvider::StonFiV2),
        url: format!("{}/{}", PROVIDER_API_URL, "stonfi_v2"),
        assets: vec![SwapChainAsset::All(Chain::Ton)],
    }
}

pub fn new_mayan() -> ProxyProvider {
    ProxyProvider {
        provider: SwapProviderType::new(SwapProvider::Mayan),
        url: format!("{}/{}", PROVIDER_API_URL, "mayan"),
        assets: vec![
            SwapChainAsset::Assets(
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
            SwapChainAsset::Assets(
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
            SwapChainAsset::Assets(Chain::Sui, vec![SUI_USDC.id.clone(), SUI_SBUSDT.id.clone(), SUI_WAL.id.clone()]),
            SwapChainAsset::Assets(
                Chain::SmartChain,
                vec![SMARTCHAIN_USDT.id.clone(), SMARTCHAIN_USDC.id.clone(), SMARTCHAIN_WBTC.id.clone()],
            ),
            SwapChainAsset::Assets(
                Chain::Base,
                vec![BASE_USDC.id.clone(), BASE_CBBTC.id.clone(), BASE_WBTC.id.clone(), BASE_USDS.id.clone()],
            ),
            SwapChainAsset::Assets(Chain::Polygon, vec![POLYGON_USDC.id.clone(), POLYGON_USDT.id.clone()]),
            SwapChainAsset::Assets(Chain::AvalancheC, vec![AVALANCHE_USDT.id.clone(), AVALANCHE_USDC.id.clone()]),
        ],
    }
}
