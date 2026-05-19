use crate::{
    QuoteRequest,
    fee_token::{FeeToken, FeeTokenPriority},
    fees::is_stablecoin_symbol,
};
use alloy_primitives::Address;
use gem_evm::uniswap::path::BasePair;

fn fee_token_priority(base_pair: Option<&BasePair>, token: &FeeToken) -> FeeTokenPriority {
    if base_pair.is_some_and(|pair| token.address == pair.native) {
        return FeeTokenPriority::Native;
    }
    if is_stablecoin_symbol(token.symbol) || base_pair.is_some_and(|pair| pair.stables.contains(&token.address)) {
        return FeeTokenPriority::Stable;
    }
    FeeTokenPriority::Other
}

pub(crate) fn is_input_fee_token(base_pair: Option<&BasePair>, input: &FeeToken, output: &FeeToken) -> bool {
    fee_token_priority(base_pair, input).rank() > fee_token_priority(base_pair, output).rank()
}

pub(crate) fn is_quote_input_fee_token(base_pair: Option<&BasePair>, request: &QuoteRequest, token_in: Address, token_out: Address) -> bool {
    let input = FeeToken::new(token_in, request.from_asset.symbol.as_str());
    let output = FeeToken::new(token_out, request.to_asset.symbol.as_str());
    is_input_fee_token(base_pair, &input, &output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::uniswap::path::get_base_pair;
    use primitives::{
        EVMChain,
        asset_constants::{ETHEREUM_UNI_TOKEN_ID, ETHEREUM_USDC_TOKEN_ID, ETHEREUM_WETH_TOKEN_ID},
    };

    #[test]
    fn test_is_input_fee_token() {
        let evm_chain = EVMChain::Ethereum;
        let base_pair = get_base_pair(&evm_chain, true);

        let weth = FeeToken::new(ETHEREUM_WETH_TOKEN_ID.parse().unwrap(), "WETH");
        let uni = FeeToken::new(ETHEREUM_UNI_TOKEN_ID.parse().unwrap(), "UNI");
        let usdc = FeeToken::new(ETHEREUM_USDC_TOKEN_ID.parse().unwrap(), "USDC");

        // WETH -> UNI (fee_token is WETH)
        assert!(is_input_fee_token(base_pair.as_ref(), &weth, &uni));

        // WETH -> USDC (fee_token is WETH)
        assert!(is_input_fee_token(base_pair.as_ref(), &weth, &usdc));

        // USDC -> WETH (fee_token is WETH)
        assert!(!is_input_fee_token(base_pair.as_ref(), &usdc, &weth));

        // USDC -> UNI (fee_token is USDC)
        assert!(is_input_fee_token(base_pair.as_ref(), &usdc, &uni));
    }

    #[test]
    fn test_is_input_fee_token_uses_stable_symbol() {
        let evm_chain = EVMChain::SmartChain;
        let v_usdt = FeeToken::new("0xfD5840Cd36d94D7229439859C0112a4185BC0255".parse().unwrap(), "vUSDT");
        let bnb_tiger = FeeToken::new("0xAc68475a88DA0fbAdB73fBF4Cc157EA137dbdC2D".parse().unwrap(), "BNBTiger");
        let base_pair = get_base_pair(&evm_chain, true);

        let native_bnb = FeeToken::new(evm_chain.weth_contract().unwrap().parse().unwrap(), "BNB");

        assert!(is_input_fee_token(base_pair.as_ref(), &v_usdt, &bnb_tiger));

        assert!(is_input_fee_token(base_pair.as_ref(), &native_bnb, &v_usdt));
    }
}
