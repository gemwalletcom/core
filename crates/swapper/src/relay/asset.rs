use std::sync::LazyLock;

use gem_evm::address::ethereum_address_checksum;
use gem_solana::{SYSTEM_PROGRAM_ID, WSOL_TOKEN_ADDRESS};
use primitives::{
    AssetId, Chain, ChainType,
    asset_constants::{
        USDC_ARB_ASSET_ID, USDC_AVAX_ASSET_ID, USDC_BASE_ASSET_ID, USDC_ETH_ASSET_ID, USDC_HYPEREVM_ASSET_ID, USDC_OP_ASSET_ID, USDC_POLYGON_ASSET_ID, USDC_SMARTCHAIN_ASSET_ID,
        USDT_ARB_ASSET_ID, USDT_AVAX_ASSET_ID, USDT_ETH_ASSET_ID, USDT_HYPEREVM_ASSET_ID, USDT_LINEA_ASSET_ID, USDT_OP_ASSET_ID, USDT_POLYGON_ASSET_ID, USDT_SMARTCHAIN_ASSET_ID,
        USDT_ZKSYNC_ASSET_ID,
    },
};

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
        SwapperChainAsset::Assets(Chain::Ethereum, vec![USDC_ETH_ASSET_ID.into(), USDT_ETH_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::SmartChain, vec![USDC_SMARTCHAIN_ASSET_ID.into(), USDT_SMARTCHAIN_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::Base, vec![USDC_BASE_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::Arbitrum, vec![USDC_ARB_ASSET_ID.into(), USDT_ARB_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::Optimism, vec![USDC_OP_ASSET_ID.into(), USDT_OP_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::Polygon, vec![USDC_POLYGON_ASSET_ID.into(), USDT_POLYGON_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::AvalancheC, vec![USDC_AVAX_ASSET_ID.into(), USDT_AVAX_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::Linea, vec![USDT_LINEA_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::ZkSync, vec![USDT_ZKSYNC_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::Hyperliquid, vec![USDC_HYPEREVM_ASSET_ID.into(), USDT_HYPEREVM_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::Berachain, vec![]),
        SwapperChainAsset::Assets(Chain::Manta, vec![]),
        SwapperChainAsset::Assets(Chain::Sonic, vec![]),
        SwapperChainAsset::Assets(Chain::Abstract, vec![]),
        SwapperChainAsset::Assets(Chain::Celo, vec![]),
        SwapperChainAsset::Assets(Chain::Stable, vec![]),
    ]
});

pub fn asset_to_currency(asset_id: &AssetId) -> Result<String, SwapperError> {
    match asset_id.chain.chain_type() {
        ChainType::Ethereum => {
            if asset_id.is_native() {
                Ok(EVM_ZERO_ADDRESS.to_string())
            } else {
                asset_id.token_id.clone().ok_or(SwapperError::NotSupportedAsset)
            }
        }
        _ => Err(SwapperError::NotSupportedChain),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_evm_native_asset() {
        let result = asset_to_currency(&AssetId::from_chain(Chain::Ethereum)).unwrap();
        assert_eq!(result, EVM_ZERO_ADDRESS);
    }

    #[test]
    fn test_evm_token_asset() {
        let token_address = ETHEREUM_USDC_TOKEN_ID;
        let result = asset_to_currency(&AssetId::from_token(Chain::Ethereum, token_address)).unwrap();
        assert_eq!(result, token_address);
    }

    #[test]
    fn test_non_evm_asset_not_supported() {
        assert_eq!(asset_to_currency(&AssetId::from_chain(Chain::Solana)), Err(SwapperError::NotSupportedChain));
    }
}
