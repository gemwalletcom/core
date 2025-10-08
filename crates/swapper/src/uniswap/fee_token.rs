use crate::SwapperMode;
use alloy_primitives::Address;
use gem_evm::uniswap::path::BasePair;
use std::collections::HashSet;

#[allow(unused)]
pub struct FeePreference {
    pub fee_token: Address,
    pub is_input_token: bool,
}

// Return (fee token, is_input_token)
pub fn get_fee_token(mode: &SwapperMode, base_pair: Option<&BasePair>, input: &Address, output: &Address) -> FeePreference {
    let use_input_as_fee_token = match mode {
        SwapperMode::ExactIn => {
            if let Some(pair) = base_pair {
                let set: HashSet<Address> = HashSet::from_iter(pair.fee_token_array());
                set.contains(input) && !set.contains(output)
            } else {
                false
            }
        }
        SwapperMode::ExactOut => true,
    };
    FeePreference {
        fee_token: if use_input_as_fee_token { *input } else { *output },
        is_input_token: use_input_as_fee_token,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;
    use gem_evm::uniswap::path::get_base_pair;
    use primitives::EVMChain;

    #[test]
    fn test_get_fee_token() {
        let evm_chain = EVMChain::Ethereum;
        let mode = SwapperMode::ExactIn;
        let base_pair = get_base_pair(&evm_chain, true);

        let weth = address!("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
        let uni = address!("0x1f9840a85d5af5bf1d1762f925bdaddc4201f984");
        let usdc = address!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
        let wbtc = address!("0x2260fac5e5542a773aa44fbcfedf7c193bc2c599");

        // WETH -> UNI (fee_token is WETH)
        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &weth, &uni);

        assert_eq!(fee_preference.fee_token, weth);
        assert!(fee_preference.is_input_token);

        // WETH -> USDC (fee_token is USDC)
        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &weth, &usdc);
        assert_eq!(fee_preference.fee_token, usdc);
        assert!(!fee_preference.is_input_token);

        // USDC -> WETH (fee_token is WETH)
        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &usdc, &weth);

        assert_eq!(fee_preference.fee_token, weth);
        assert!(!fee_preference.is_input_token);

        // USDC -> UNI (fee_token is USDC)
        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &usdc, &uni);

        assert_eq!(fee_preference.fee_token, usdc);
        assert!(fee_preference.is_input_token);

        // WBTC -> UNI (fee_token is WBTC)

        let fee_preference = get_fee_token(&mode, base_pair.as_ref(), &wbtc, &uni);

        assert_eq!(fee_preference.fee_token, wbtc);
        assert!(fee_preference.is_input_token);
    }
}
