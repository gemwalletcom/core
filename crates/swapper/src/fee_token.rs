use crate::fees::is_stablecoin_symbol;
use alloy_primitives::Address;
use primitives::{AssetId, Chain, EVMChain, contract_constants::SOLANA_WRAPPED_SOL_TOKEN_ADDRESS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FeeTokenPriority {
    Native,
    Stable,
    Other,
}

pub(crate) struct FeeToken<'a> {
    pub address: Address,
    pub symbol: &'a str,
}

impl<'a> FeeToken<'a> {
    pub(crate) fn new(address: Address, symbol: &'a str) -> Self {
        Self { address, symbol }
    }
}

impl FeeTokenPriority {
    pub(crate) fn from_asset(asset_id: &AssetId, symbol: &str) -> Self {
        if asset_id.is_native() || is_wrapped_native_token(asset_id) {
            return Self::Native;
        }
        if is_stablecoin_symbol(symbol) {
            return Self::Stable;
        }
        Self::Other
    }

    pub(crate) fn rank(self) -> u8 {
        match self {
            Self::Native => 3,
            Self::Stable => 2,
            Self::Other => 1,
        }
    }
}

fn is_wrapped_native_token(asset_id: &AssetId) -> bool {
    let Some(token_id) = asset_id.token_id.as_deref() else {
        return false;
    };
    if asset_id.chain == Chain::Solana {
        return token_id == SOLANA_WRAPPED_SOL_TOKEN_ADDRESS;
    }
    let Some(chain) = EVMChain::from_chain(asset_id.chain) else {
        return false;
    };
    chain.weth_contract().is_some_and(|wrapped| token_id.eq_ignore_ascii_case(wrapped))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{
        asset_constants::{SMARTCHAIN_CAKE_ASSET_ID, SMARTCHAIN_USDC_ASSET_ID},
        contract_constants::SOLANA_WRAPPED_SOL_TOKEN_ADDRESS,
    };

    #[test]
    fn test_fee_token_priority_from_asset() {
        let bnb = AssetId::from_chain(Chain::SmartChain);
        let wbnb = AssetId::from_token(Chain::SmartChain, EVMChain::SmartChain.weth_contract().unwrap());
        let usdc = SMARTCHAIN_USDC_ASSET_ID.clone();
        let cake = SMARTCHAIN_CAKE_ASSET_ID.clone();
        let wsol = AssetId::from_token(Chain::Solana, SOLANA_WRAPPED_SOL_TOKEN_ADDRESS);

        assert_eq!(FeeTokenPriority::from_asset(&bnb, "BNB"), FeeTokenPriority::Native);
        assert_eq!(FeeTokenPriority::from_asset(&wbnb, "WBNB"), FeeTokenPriority::Native);
        assert_eq!(FeeTokenPriority::from_asset(&usdc, "USDC"), FeeTokenPriority::Stable);
        assert_eq!(FeeTokenPriority::from_asset(&cake, "CAKE"), FeeTokenPriority::Other);
        assert_eq!(FeeTokenPriority::from_asset(&wsol, "SOL"), FeeTokenPriority::Native);
    }

    #[test]
    fn test_fee_token_priority_ordering() {
        assert!(FeeTokenPriority::Native.rank() > FeeTokenPriority::Stable.rank());
        assert!(FeeTokenPriority::Stable.rank() > FeeTokenPriority::Other.rank());
    }
}
