use std::sync::LazyLock;

use gem_evm::address::ethereum_address_checksum;
use gem_solana::{SYSTEM_PROGRAM_ID, WSOL_TOKEN_ADDRESS};
use primitives::{
    AssetId, Chain, ChainType,
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

pub fn asset_to_currency(asset_id: &AssetId, relay_chain: &RelayChain) -> Result<String, SwapperError> {
    match relay_chain {
        RelayChain::Bitcoin => Ok(BITCOIN_CURRENCY.to_string()),
        RelayChain::Solana => {
            if asset_id.is_native() {
                Ok(SYSTEM_PROGRAM_ID.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_evm_native_asset() {
        let result = asset_to_currency(&AssetId::from_chain(Chain::Ethereum), &RelayChain::Ethereum).unwrap();
        assert_eq!(result, EVM_ZERO_ADDRESS);
    }

    #[test]
    fn test_evm_token_asset() {
        let token_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let result = asset_to_currency(&AssetId::from_token(Chain::Ethereum, token_address), &RelayChain::Ethereum).unwrap();
        assert_eq!(result, token_address);
    }

    #[test]
    fn test_solana_native_asset() {
        let result = asset_to_currency(&AssetId::from_chain(Chain::Solana), &RelayChain::Solana).unwrap();
        assert_eq!(result, SYSTEM_PROGRAM_ID);
    }

    #[test]
    fn test_solana_token_asset() {
        let mint_address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let result = asset_to_currency(&AssetId::from_token(Chain::Solana, mint_address), &RelayChain::Solana).unwrap();
        assert_eq!(result, mint_address);
    }
}
