use std::sync::LazyLock;

use gem_evm::address::ethereum_address_checksum;
use primitives::{
    AssetId, Chain,
    asset_constants::{
        USDC_ARB_ASSET_ID, USDC_HYPEREVM_ASSET_ID, USDC_OP_ASSET_ID, USDC_POLYGON_ASSET_ID, USDT_ARB_ASSET_ID, USDT_HYPEREVM_ASSET_ID, USDT_LINEA_ASSET_ID, USDT_OP_ASSET_ID,
        USDT_POLYGON_ASSET_ID, USDT_ZKSYNC_ASSET_ID,
    },
};

use super::chain::{BITCOIN_CURRENCY, RelayChain};
use crate::{SwapperChainAsset, SwapperError, asset::*};

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
        SwapperChainAsset::Assets(
            Chain::Ethereum,
            vec![
                AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
                AssetId::from_token(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID),
            ],
        ),
        SwapperChainAsset::Assets(
            Chain::SmartChain,
            vec![
                AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID),
                AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID),
            ],
        ),
        SwapperChainAsset::Assets(Chain::Base, vec![AssetId::from_token(Chain::Base, BASE_USDC_TOKEN_ID)]),
        SwapperChainAsset::Assets(Chain::Arbitrum, vec![USDC_ARB_ASSET_ID.into(), USDT_ARB_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::Optimism, vec![USDC_OP_ASSET_ID.into(), USDT_OP_ASSET_ID.into()]),
        SwapperChainAsset::Assets(Chain::Polygon, vec![USDC_POLYGON_ASSET_ID.into(), USDT_POLYGON_ASSET_ID.into()]),
        SwapperChainAsset::Assets(
            Chain::AvalancheC,
            vec![
                AssetId::from_token(Chain::AvalancheC, AVALANCHE_USDC_TOKEN_ID),
                AssetId::from_token(Chain::AvalancheC, AVALANCHE_USDT_TOKEN_ID),
            ],
        ),
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
    use primitives::Chain;

    #[test]
    fn test_evm_native_asset() {
        let result = asset_to_currency(&AssetId::from_chain(Chain::Ethereum), &RelayChain::Evm(primitives::chain_evm::EVMChain::Ethereum)).unwrap();
        assert_eq!(result, EVM_ZERO_ADDRESS);
    }

    #[test]
    fn test_evm_token_asset() {
        let token_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
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
