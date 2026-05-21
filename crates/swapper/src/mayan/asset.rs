use crate::SwapperChainAsset;
use gem_evm::{EVM_ZERO_ADDRESS, ethereum_address_checksum};
use gem_solana::WSOL_TOKEN_ADDRESS;
use gem_sui::SUI_COIN_TYPE;
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

use super::constants::HYPERCORE_SPOT_USDC_CONTRACT;

pub fn supported_assets() -> Vec<SwapperChainAsset> {
    vec![
        SwapperChainAsset::assets(
            Chain::Ethereum,
            [
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
        SwapperChainAsset::assets(
            Chain::Solana,
            [
                SOLANA_USDC_ASSET_ID.clone(),
                SOLANA_USDT_ASSET_ID.clone(),
                SOLANA_USDS_ASSET_ID.clone(),
                SOLANA_CBBTC_ASSET_ID.clone(),
                SOLANA_WBTC_ASSET_ID.clone(),
                SOLANA_JITO_SOL_ASSET_ID.clone(),
            ],
        ),
        SwapperChainAsset::assets(Chain::Sui, [SUI_USDC_ASSET_ID.clone(), SUI_SBUSDT_ASSET_ID.clone(), SUI_WAL_ASSET_ID.clone()]),
        SwapperChainAsset::assets(
            Chain::SmartChain,
            [SMARTCHAIN_USDT_ASSET_ID.clone(), SMARTCHAIN_USDC_ASSET_ID.clone(), SMARTCHAIN_WBTC_ASSET_ID.clone()],
        ),
        SwapperChainAsset::assets(
            Chain::Base,
            [
                BASE_USDC_ASSET_ID.clone(),
                BASE_CBBTC_ASSET_ID.clone(),
                BASE_WBTC_ASSET_ID.clone(),
                BASE_USDS_ASSET_ID.clone(),
            ],
        ),
        SwapperChainAsset::assets(Chain::Polygon, [POLYGON_USDC_ASSET_ID.clone(), POLYGON_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::assets(Chain::AvalancheC, [AVALANCHE_USDT_ASSET_ID.clone(), AVALANCHE_USDC_ASSET_ID.clone()]),
        SwapperChainAsset::assets(Chain::Arbitrum, [ARBITRUM_USDC_ASSET_ID.clone(), ARBITRUM_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::assets(Chain::Optimism, [OPTIMISM_USDC_ASSET_ID.clone(), OPTIMISM_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::assets(Chain::Linea, [LINEA_USDC_E_ASSET_ID.clone(), LINEA_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::assets(Chain::Unichain, [UNICHAIN_USDC_ASSET_ID.clone(), UNICHAIN_DAI_ASSET_ID.clone()]),
        SwapperChainAsset::assets(Chain::Monad, [Chain::Monad.as_asset_id(), MONAD_USDC_ASSET_ID.clone(), MONAD_USDT_ASSET_ID.clone()]),
        SwapperChainAsset::assets(Chain::Hyperliquid, [HYPEREVM_USDT_ASSET_ID.clone(), HYPEREVM_USDC_ASSET_ID.clone()]),
        SwapperChainAsset::assets(Chain::HyperCore, [HYPERCORE_SPOT_USDC_ASSET_ID.clone()]),
    ]
}

pub fn token_id_for_asset(asset_id: &AssetId) -> String {
    match (asset_id.chain, asset_id.token_id.as_deref()) {
        (Chain::Sui, None) => SUI_COIN_TYPE.to_string(),
        (Chain::HyperCore, Some(HYPERCORE_SPOT_USDC_TOKEN_ID)) => HYPERCORE_SPOT_USDC_CONTRACT.to_string(),
        (_, None) => EVM_ZERO_ADDRESS.to_string(),
        (_, Some(token_id)) => token_id.to_string(),
    }
}

pub fn asset_id_for_token(chain: Chain, token_address: &str) -> Option<AssetId> {
    match chain {
        Chain::Solana => match token_address {
            EVM_ZERO_ADDRESS | WSOL_TOKEN_ADDRESS => Some(AssetId::from_chain(chain)),
            _ => Some(AssetId::from_token(chain, token_address)),
        },
        Chain::Sui => match token_address {
            SUI_COIN_TYPE => Some(AssetId::from_chain(chain)),
            _ => Some(AssetId::from_token(chain, token_address)),
        },
        Chain::HyperCore => {
            if token_address.eq_ignore_ascii_case(HYPERCORE_SPOT_USDC_CONTRACT) {
                Some(HYPERCORE_SPOT_USDC_ASSET_ID.clone())
            } else {
                Some(AssetId::from_token(chain, token_address))
            }
        }
        _ if chain.chain_type() == ChainType::Ethereum => match token_address {
            EVM_ZERO_ADDRESS => Some(AssetId::from_chain(chain)),
            _ => ethereum_address_checksum(token_address).ok().map(|address| AssetId::from_token(chain, &address)),
        },
        _ => match chain.as_denom() {
            Some(denom) if denom == token_address => Some(AssetId::from_chain(chain)),
            _ => Some(AssetId::from_token(chain, token_address)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::ETHEREUM_USDC_ASSET_ID;

    #[test]
    fn test_asset_id_for_token() {
        assert_eq!(asset_id_for_token(Chain::Ethereum, EVM_ZERO_ADDRESS), Some(AssetId::from_chain(Chain::Ethereum)));
        assert_eq!(asset_id_for_token(Chain::Sui, SUI_COIN_TYPE), Some(AssetId::from_chain(Chain::Sui)));
        assert_eq!(asset_id_for_token(Chain::Solana, EVM_ZERO_ADDRESS), Some(AssetId::from_chain(Chain::Solana)));
        assert_eq!(asset_id_for_token(Chain::Solana, WSOL_TOKEN_ADDRESS), Some(AssetId::from_chain(Chain::Solana)));
        assert_eq!(
            asset_id_for_token(Chain::Ethereum, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
            Some(ETHEREUM_USDC_ASSET_ID.clone())
        );
        assert_eq!(
            asset_id_for_token(Chain::HyperCore, HYPERCORE_SPOT_USDC_CONTRACT),
            Some(HYPERCORE_SPOT_USDC_ASSET_ID.clone())
        );
    }

    #[test]
    fn test_token_id_for_asset() {
        assert_eq!(token_id_for_asset(&AssetId::from_chain(Chain::Ethereum)), EVM_ZERO_ADDRESS);
        assert_eq!(token_id_for_asset(&AssetId::from_chain(Chain::Solana)), EVM_ZERO_ADDRESS);
        assert_eq!(token_id_for_asset(&AssetId::from_chain(Chain::Sui)), SUI_COIN_TYPE);
        assert_eq!(token_id_for_asset(&HYPERCORE_SPOT_USDC_ASSET_ID), HYPERCORE_SPOT_USDC_CONTRACT);
        assert_eq!(token_id_for_asset(&ETHEREUM_USDC_ASSET_ID), ETHEREUM_USDC_ASSET_ID.token_id.clone().unwrap());
    }
}
