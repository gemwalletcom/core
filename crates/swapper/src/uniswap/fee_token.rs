use crate::{SwapperMode, fees::is_stable_symbol};
use alloy_primitives::Address;
use gem_evm::uniswap::path::BasePair;
use std::collections::HashSet;

#[allow(unused)]
pub struct FeePreference {
    pub fee_token: Address,
    pub is_input_token: bool,
}

pub struct FeeToken<'a> {
    pub address: Address,
    pub symbol: &'a str,
}

impl<'a> FeeToken<'a> {
    pub fn new(address: Address, symbol: &'a str) -> Self {
        Self { address, symbol }
    }
}

fn is_preferred_fee_token(base_pair: Option<&BasePair>, token: &FeeToken) -> bool {
    if is_stable_symbol(token.symbol) {
        return true;
    }

    match base_pair {
        Some(pair) => {
            let set: HashSet<Address> = HashSet::from_iter(pair.fee_token_array());
            set.contains(&token.address)
        }
        None => false,
    }
}

// Return (fee token, is_input_token)
pub fn get_fee_token(mode: &SwapperMode, base_pair: Option<&BasePair>, input: &FeeToken, output: &FeeToken) -> FeePreference {
    let use_input_as_fee_token = match mode {
        SwapperMode::ExactIn => is_preferred_fee_token(base_pair, input) && !is_preferred_fee_token(base_pair, output),
        SwapperMode::ExactOut => true,
    };
    FeePreference {
        fee_token: if use_input_as_fee_token { input.address } else { output.address },
        is_input_token: use_input_as_fee_token,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::uniswap::path::get_base_pair;
    use primitives::{
        EVMChain,
        asset_constants::{ETHEREUM_UNI_TOKEN_ID, ETHEREUM_USDC_TOKEN_ID, ETHEREUM_WBTC_TOKEN_ID, ETHEREUM_WETH_TOKEN_ID},
    };

    #[test]
    fn test_get_fee_token() {
        let evm_chain = EVMChain::Ethereum;
        let mode = SwapperMode::ExactIn;
        let base_pair = get_base_pair(&evm_chain, true);

        let weth = FeeToken::new(ETHEREUM_WETH_TOKEN_ID.parse().unwrap(), "WETH");
        let uni = FeeToken::new(ETHEREUM_UNI_TOKEN_ID.parse().unwrap(), "UNI");
        let usdc = FeeToken::new(ETHEREUM_USDC_TOKEN_ID.parse().unwrap(), "USDC");
        let wbtc = FeeToken::new(ETHEREUM_WBTC_TOKEN_ID.parse().unwrap(), "WBTC");

        // WETH -> UNI (fee_token is WETH)
        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &weth, &uni);

        assert_eq!(fee_preference.fee_token, weth.address);
        assert!(fee_preference.is_input_token);

        // WETH -> USDC (fee_token is USDC)
        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &weth, &usdc);
        assert_eq!(fee_preference.fee_token, usdc.address);
        assert!(!fee_preference.is_input_token);

        // USDC -> WETH (fee_token is WETH)
        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &usdc, &weth);

        assert_eq!(fee_preference.fee_token, weth.address);
        assert!(!fee_preference.is_input_token);

        // USDC -> UNI (fee_token is USDC)
        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &usdc, &uni);

        assert_eq!(fee_preference.fee_token, usdc.address);
        assert!(fee_preference.is_input_token);

        // WBTC -> UNI (fee_token is WBTC)

        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &wbtc, &uni);

        assert_eq!(fee_preference.fee_token, wbtc.address);
        assert!(fee_preference.is_input_token);
    }

    #[test]
    fn test_get_fee_token_uses_stable_symbol() {
        let evm_chain = EVMChain::SmartChain;
        let mode = SwapperMode::ExactIn;
        let v_usdt = FeeToken::new("0xfD5840Cd36d94D7229439859C0112a4185BC0255".parse().unwrap(), "vUSDT");
        let bnb_tiger = FeeToken::new("0xAc68475a88DA0fbAdB73fBF4Cc157EA137dbdC2D".parse().unwrap(), "BNBTiger");
        let base_pair = get_base_pair(&evm_chain, true);

        let native_bnb = FeeToken::new(evm_chain.weth_contract().unwrap().parse().unwrap(), "BNB");

        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &v_usdt, &bnb_tiger);
        assert_eq!(fee_preference.fee_token, v_usdt.address);
        assert!(fee_preference.is_input_token);

        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &native_bnb, &v_usdt);
        assert_eq!(fee_preference.fee_token, v_usdt.address);
        assert!(!fee_preference.is_input_token);
    }
}
