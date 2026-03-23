use std::sync::LazyLock;

use crate::{SwapperChainAsset, SwapperError};
use gem_evm::address::ethereum_address_checksum;
use primitives::{
    AssetId, Chain,
    asset_constants::{
        ARBITRUM_USDC_ASSET_ID, ARBITRUM_USDT_ASSET_ID, AVALANCHE_USDC_ASSET_ID, AVALANCHE_USDT_ASSET_ID, BASE_USDC_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID,
        HYPEREVM_USDC_ASSET_ID, HYPEREVM_USDT_ASSET_ID, LINEA_USDT_ASSET_ID, OPTIMISM_USDC_ASSET_ID, OPTIMISM_USDT_ASSET_ID, POLYGON_USDC_ASSET_ID, POLYGON_USDT_ASSET_ID,
        SMARTCHAIN_USDC_ASSET_ID, SMARTCHAIN_USDT_ASSET_ID, ZKSYNC_USDT_ASSET_ID,
    },
    contract_constants::EVM_ZERO_ADDRESS,
};

use super::chain::{BITCOIN_CURRENCY, RelayChain};

pub fn map_currency_to_asset_id(relay_chain: RelayChain, currency: &str) -> AssetId {
    let chain = relay_chain.to_chain();
    match relay_chain {
        RelayChain::Bitcoin => AssetId::from_chain(chain),
        RelayChain::Evm(_) => {
            if currency == EVM_ZERO_ADDRESS {
                AssetId::from_chain(chain)
            } else {
                let address = ethereum_address_checksum(currency).unwrap_or(currency.to_string());
                AssetId::from_token(chain, &address)
            }
        }
    }
}

pub static SUPPORTED_CHAINS: LazyLock<Vec<SwapperChainAsset>> = LazyLock::new(|| {
    vec![
        SwapperChainAsset::Assets(Chain::Bitcoin, vec![AssetId::from_chain(Chain::Bitcoin)]),
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
        SwapperChainAsset::Assets(Chain::Berachain, vec![]),
        SwapperChainAsset::Assets(Chain::Manta, vec![]),
        SwapperChainAsset::Assets(Chain::Sonic, vec![]),
        SwapperChainAsset::Assets(Chain::Abstract, vec![]),
        SwapperChainAsset::Assets(Chain::Celo, vec![]),
        SwapperChainAsset::Assets(Chain::Stable, vec![]),
    ]
});

pub fn asset_to_currency(asset_id: &AssetId, relay_chain: &RelayChain) -> Result<String, SwapperError> {
    match relay_chain {
        RelayChain::Bitcoin => Ok(BITCOIN_CURRENCY.to_string()),
        RelayChain::Evm(_) => {
            if asset_id.is_native() {
                Ok(EVM_ZERO_ADDRESS.to_string())
            } else {
                asset_id.token_id.clone().ok_or(SwapperError::NotSupportedAsset)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, asset_constants::ETHEREUM_USDC_TOKEN_ID};

    #[test]
    fn test_evm_native_asset() {
        let result = asset_to_currency(&AssetId::from_chain(Chain::Ethereum), &RelayChain::Evm(primitives::chain_evm::EVMChain::Ethereum)).unwrap();
        assert_eq!(result, EVM_ZERO_ADDRESS);
    }

    #[test]
    fn test_evm_token_asset() {
        let token_address = ETHEREUM_USDC_TOKEN_ID;
        let result = asset_to_currency(
            &AssetId::from_token(Chain::Ethereum, token_address),
            &RelayChain::Evm(primitives::chain_evm::EVMChain::Ethereum),
        )
        .unwrap();
        assert_eq!(result, token_address);
    }

    #[test]
    fn test_bitcoin_asset() {
        let result = asset_to_currency(&AssetId::from_chain(Chain::Bitcoin), &RelayChain::Bitcoin).unwrap();
        assert_eq!(result, BITCOIN_CURRENCY);
    }

    #[test]
    fn test_non_supported_chain() {
        // RelayChain can't represent Solana, so this is tested at the from_chain level
        assert!(RelayChain::from_chain(&Chain::Solana).is_none());
    }
}
