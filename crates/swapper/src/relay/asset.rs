use std::sync::LazyLock;

use gem_solana::WSOL_TOKEN_ADDRESS;
use primitives::{
    AssetId, Chain,
    asset_constants::{USDC_ARB_ASSET_ID, USDC_HYPEREVM_ASSET_ID, USDT_ARB_ASSET_ID, USDT_HYPEREVM_ASSET_ID},
};

use super::chain::{BITCOIN_CURRENCY, RelayChain};
use crate::{SwapperChainAsset, SwapperError, asset::*};

pub static SUPPORTED_CHAINS: LazyLock<Vec<SwapperChainAsset>> = LazyLock::new(|| {
    vec![
        SwapperChainAsset::Assets(Chain::Bitcoin, vec![AssetId::from_chain(Chain::Bitcoin)]),
        SwapperChainAsset::Assets(
            Chain::Ethereum,
            vec![
                AssetId::from_chain(Chain::Ethereum),
                AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
                AssetId::from_token(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID),
            ],
        ),
        SwapperChainAsset::Assets(
            Chain::Solana,
            vec![
                AssetId::from_chain(Chain::Solana),
                AssetId::from_token(Chain::Solana, SOLANA_USDC_TOKEN_ID),
                AssetId::from_token(Chain::Solana, SOLANA_USDT_TOKEN_ID),
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
            Chain::Hyperliquid,
            vec![AssetId::from_chain(Chain::Hyperliquid), USDC_HYPEREVM_ASSET_ID.into(), USDT_HYPEREVM_ASSET_ID.into()],
        ),
        SwapperChainAsset::Assets(Chain::Berachain, vec![AssetId::from_chain(Chain::Berachain)]),
        SwapperChainAsset::Assets(Chain::Manta, vec![AssetId::from_chain(Chain::Manta)]),
        SwapperChainAsset::Assets(Chain::Sonic, vec![AssetId::from_chain(Chain::Sonic)]),
        SwapperChainAsset::Assets(Chain::Abstract, vec![AssetId::from_chain(Chain::Abstract)]),
        SwapperChainAsset::Assets(Chain::Mantle, vec![AssetId::from_chain(Chain::Mantle)]),
        SwapperChainAsset::Assets(Chain::Celo, vec![AssetId::from_chain(Chain::Celo)]),
        SwapperChainAsset::Assets(Chain::Stable, vec![AssetId::from_chain(Chain::Stable)]),
    ]
});

pub fn map_asset_to_relay_currency(asset_id: &AssetId, relay_chain: &RelayChain) -> Result<String, SwapperError> {
    match relay_chain {
        RelayChain::Bitcoin => Ok(BITCOIN_CURRENCY.to_string()),
        RelayChain::Solana => {
            if asset_id.is_native() {
                Ok(WSOL_TOKEN_ADDRESS.to_string())
            } else {
                asset_id.token_id.clone().ok_or(SwapperError::NotSupportedAsset)
            }
        }
        _ if relay_chain.is_evm() => {
            if asset_id.is_native() {
                Ok(EVM_ZERO_ADDRESS.to_string())
            } else {
                asset_id.token_id.clone().ok_or(SwapperError::NotSupportedAsset)
            }
        }
        _ => Err(SwapperError::NotSupportedChain),
    }
}

pub fn relay_currency_to_asset_id(chain: Chain, currency: &str) -> AssetId {
    match chain {
        Chain::Bitcoin => AssetId::from_chain(Chain::Bitcoin),
        Chain::Solana if currency == WSOL_TOKEN_ADDRESS => AssetId::from_chain(Chain::Solana),
        _ if currency == EVM_ZERO_ADDRESS => AssetId::from_chain(chain),
        _ => AssetId::from_token(chain, currency),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_evm_native_asset() {
        let asset_id = AssetId::from_chain(Chain::Ethereum);
        let relay_chain = RelayChain::Ethereum;
        let result = map_asset_to_relay_currency(&asset_id, &relay_chain).unwrap();
        assert_eq!(result, EVM_ZERO_ADDRESS);
    }

    #[test]
    fn test_evm_token_asset() {
        let token_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let asset_id = AssetId::from_token(Chain::Ethereum, token_address);
        let relay_chain = RelayChain::Ethereum;
        let result = map_asset_to_relay_currency(&asset_id, &relay_chain).unwrap();
        assert_eq!(result, token_address);
    }

    #[test]
    fn test_solana_native_asset() {
        let asset_id = AssetId::from_chain(Chain::Solana);
        let relay_chain = RelayChain::Solana;
        let result = map_asset_to_relay_currency(&asset_id, &relay_chain).unwrap();
        assert_eq!(result, WSOL_TOKEN_ADDRESS);
    }

    #[test]
    fn test_solana_token_asset() {
        let mint_address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let asset_id = AssetId::from_token(Chain::Solana, mint_address);
        let relay_chain = RelayChain::Solana;
        let result = map_asset_to_relay_currency(&asset_id, &relay_chain).unwrap();
        assert_eq!(result, mint_address);
    }
}
