use std::sync::LazyLock;

use gem_evm::address::ethereum_address_checksum;
use gem_solana::{SYSTEM_PROGRAM_ID, WSOL_TOKEN_ADDRESS};
use primitives::{
    AssetId, Chain, ChainType,
    asset_constants::{
        USDC_ARB_ASSET_ID, USDC_HYPEREVM_ASSET_ID, USDC_OP_ASSET_ID, USDC_POLYGON_ASSET_ID, USDT_ARB_ASSET_ID, USDT_HYPEREVM_ASSET_ID, USDT_LINEA_ASSET_ID, USDT_OP_ASSET_ID,
        USDT_POLYGON_ASSET_ID, USDT_ZKSYNC_ASSET_ID,
    },
};

use super::chain::RelayChain;
use crate::{SwapperChainAsset, SwapperError, asset::*};

fn is_native_currency(chain: Chain, currency: &str) -> bool {
    match chain {
        Chain::Bitcoin => true,
        Chain::Solana => currency == SYSTEM_PROGRAM_ID || currency == WSOL_TOKEN_ADDRESS,
        _ if currency == EVM_ZERO_ADDRESS => true,
        _ => false,
    }
}

pub fn map_currency_to_asset_id(chain: Chain, currency: &str) -> AssetId {
    if is_native_currency(chain, currency) {
        return AssetId::from_chain(chain);
    }
    if let ChainType::Ethereum = chain.chain_type()
        && let Ok(address) = ethereum_address_checksum(currency)
    {
        return AssetId::from_token(chain, &address);
    }
    AssetId::from_token(chain, currency)
}

pub static SUPPORTED_CHAINS: LazyLock<Vec<SwapperChainAsset>> = LazyLock::new(|| {
    vec![
        SwapperChainAsset::Assets(
            Chain::Ethereum,
            vec![
                AssetId::from_chain(Chain::Ethereum),
                AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
                AssetId::from_token(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID),
            ],
        ),
        SwapperChainAsset::Assets(
            Chain::SmartChain,
            vec![
                AssetId::from_chain(Chain::SmartChain),
                AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID),
                AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID),
            ],
        ),
        SwapperChainAsset::Assets(Chain::Base, vec![AssetId::from_chain(Chain::Base), AssetId::from_token(Chain::Base, BASE_USDC_TOKEN_ID)]),
        SwapperChainAsset::Assets(
            Chain::Arbitrum,
            vec![AssetId::from_chain(Chain::Arbitrum), USDC_ARB_ASSET_ID.into(), USDT_ARB_ASSET_ID.into()],
        ),
        SwapperChainAsset::Assets(
            Chain::Optimism,
            vec![AssetId::from_chain(Chain::Optimism), USDC_OP_ASSET_ID.into(), USDT_OP_ASSET_ID.into()],
        ),
        SwapperChainAsset::Assets(
            Chain::Polygon,
            vec![AssetId::from_chain(Chain::Polygon), USDC_POLYGON_ASSET_ID.into(), USDT_POLYGON_ASSET_ID.into()],
        ),
        SwapperChainAsset::Assets(
            Chain::AvalancheC,
            vec![
                AssetId::from_chain(Chain::AvalancheC),
                AssetId::from_token(Chain::AvalancheC, AVALANCHE_USDC_TOKEN_ID),
                AssetId::from_token(Chain::AvalancheC, AVALANCHE_USDT_TOKEN_ID),
            ],
        ),
        SwapperChainAsset::Assets(Chain::Linea, vec![AssetId::from_chain(Chain::Linea), USDT_LINEA_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::ZkSync, vec![AssetId::from_chain(Chain::ZkSync), USDT_ZKSYNC_ASSET_ID.into()]),
        SwapperChainAsset::Assets(
            Chain::Hyperliquid,
            vec![AssetId::from_chain(Chain::Hyperliquid), USDC_HYPEREVM_ASSET_ID.into(), USDT_HYPEREVM_ASSET_ID.into()],
        ),
        SwapperChainAsset::Assets(Chain::Berachain, vec![]),
        SwapperChainAsset::Assets(Chain::Abstract, vec![]),
        SwapperChainAsset::Assets(Chain::Mantle, vec![]),
        SwapperChainAsset::Assets(Chain::Celo, vec![]),
        SwapperChainAsset::Assets(Chain::Stable, vec![]),
    ]
});

pub fn asset_to_currency(asset_id: &AssetId, relay_chain: &RelayChain) -> Result<String, SwapperError> {
    if !relay_chain.is_evm() {
        return Err(SwapperError::NotSupportedChain);
    }
    if asset_id.is_native() {
        Ok(EVM_ZERO_ADDRESS.to_string())
    } else {
        asset_id.token_id.clone().ok_or(SwapperError::NotSupportedAsset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_evm_native_asset() {
        let result = asset_to_currency(&AssetId::from_chain(Chain::Ethereum), &RelayChain::from_chain(&Chain::Ethereum).unwrap()).unwrap();
        assert_eq!(result, EVM_ZERO_ADDRESS);
    }

    #[test]
    fn test_evm_token_asset() {
        let token_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let result = asset_to_currency(&AssetId::from_token(Chain::Ethereum, token_address), &RelayChain::from_chain(&Chain::Ethereum).unwrap()).unwrap();
        assert_eq!(result, token_address);
    }
}
