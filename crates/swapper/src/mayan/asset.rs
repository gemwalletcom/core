use gem_evm::{EVM_ZERO_ADDRESS, ethereum_address_checksum};
use gem_solana::WSOL_TOKEN_ADDRESS;
use gem_sui::SUI_COIN_TYPE;
use primitives::{AssetId, Chain, ChainType, asset_constants::HYPERCORE_SPOT_USDC_ASSET_ID};

use super::constants::HYPERCORE_SPOT_USDC_CONTRACT;

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
}
