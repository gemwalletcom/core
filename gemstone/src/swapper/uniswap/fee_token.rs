use crate::swapper::GemSwapMode;
use alloy_primitives::Address;
use gem_evm::uniswap::path::BasePair;

#[allow(unused)]
pub struct FeePreference {
    pub fee_token: Address,
    pub is_input_token: bool,
}

// Return (fee token, is_input_token)
pub fn get_fee_token(mode: &GemSwapMode, base_pair: Option<&BasePair>, input: &Address, output: &Address) -> FeePreference {
    let use_input_as_fee_token = match mode {
        GemSwapMode::ExactIn => {
            if let Some(pair) = base_pair {
                let set = pair.to_set();
                set.contains(input) && !set.contains(output)
            } else {
                false
            }
        }
        GemSwapMode::ExactOut => true,
    };
    FeePreference {
        fee_token: if use_input_as_fee_token { *input } else { *output },
        is_input_token: use_input_as_fee_token,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swapper::GemSwapMode;
    use gem_evm::uniswap::path::get_base_pair;
    use primitives::EVMChain;
    use std::str::FromStr;

    #[test]
    fn test_get_fee_token() {
        let evm_chain = EVMChain::Ethereum;
        let mode = GemSwapMode::ExactIn;
        let base_pair = get_base_pair(&evm_chain, true);

        let weth = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
        let uni = Address::from_str("0x1f9840a85d5af5bf1d1762f925bdaddc4201f984").unwrap();
        let usdc = Address::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();

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
    }
}
