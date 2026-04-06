use std::sync::LazyLock;

use crate::{SwapperChainAsset, SwapperError};
use gem_evm::address::ethereum_address_checksum;
use gem_solana::{SYSTEM_PROGRAM_ID, WSOL_TOKEN_ADDRESS};
use primitives::{
    AssetId, Chain, ChainType,
    asset_constants::{
        ARBITRUM_USDC_ASSET_ID, ARBITRUM_USDT_ASSET_ID, AVALANCHE_USDC_ASSET_ID, AVALANCHE_USDT_ASSET_ID, BASE_USDC_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID,
        HYPEREVM_USDC_ASSET_ID, HYPEREVM_USDT_ASSET_ID, LINEA_USDT_ASSET_ID, OPTIMISM_USDC_ASSET_ID, OPTIMISM_USDT_ASSET_ID, POLYGON_USDC_ASSET_ID, POLYGON_USDT_ASSET_ID,
        SEIEVM_USDC_N_ASSET_ID, SMARTCHAIN_USDC_ASSET_ID, SMARTCHAIN_USDT_ASSET_ID, ZKSYNC_USDT_ASSET_ID,
    },
    contract_constants::EVM_ZERO_ADDRESS,
};

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
        SwapperChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_USDC_ASSET_ID.clone(), ETHEREUM_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::Assets(Chain::SmartChain, vec![SMARTCHAIN_USDC_ASSET_ID.clone(), SMARTCHAIN_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::Assets(Chain::Base, vec![BASE_USDC_ASSET_ID.clone()]),
        SwapperChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDC_ASSET_ID.clone(), ARBITRUM_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_USDC_ASSET_ID.clone(), OPTIMISM_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::Assets(Chain::Polygon, vec![POLYGON_USDC_ASSET_ID.clone(), POLYGON_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::Assets(Chain::AvalancheC, vec![AVALANCHE_USDC_ASSET_ID.clone(), AVALANCHE_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::Assets(Chain::Linea, vec![LINEA_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::Assets(Chain::ZkSync, vec![ZKSYNC_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::Assets(Chain::Hyperliquid, vec![HYPEREVM_USDC_ASSET_ID.clone(), HYPEREVM_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::Assets(Chain::SeiEvm, vec![SEIEVM_USDC_N_ASSET_ID.clone()]),
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
    use primitives::{Chain, asset_constants::ETHEREUM_USDC_TOKEN_ID};

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
