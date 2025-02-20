use crate::{network::JsonRpcResponse, swapper::SwapperError};
use alloy_core::{hex, primitives::Address, sol_types::SolCall};
use alloy_primitives::{Bytes, U256};
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    uniswap::{
        contracts::v4::{IV4Quoter, PoolKey},
        path::TokenPair,
    },
};

pub fn build_quote_exact_single_request(token_in: &EthereumAddress, v4_quoter: &str, amount_in: u128, pool: &PoolKey) -> EthereumRpc {
    let token_in_address = Address::from_slice(&token_in.bytes);
    let zero_for_one = token_in_address == pool.currency0;
    let params = IV4Quoter::QuoteExactSingleParams {
        poolKey: pool.clone(),
        zeroForOne: zero_for_one,
        exactAmount: amount_in,
        hookData: Bytes::new(),
    };
    let quote_single = IV4Quoter::quoteExactInputSingleCall { params };
    let call_data: Vec<u8> = quote_single.abi_encode();
    EthereumRpc::Call(TransactionObject::new_call(v4_quoter, call_data), BlockParameter::Latest)
}

pub fn build_quote_exact_requests(v4_quoter: &str, quote_params: &[Vec<(Vec<TokenPair>, IV4Quoter::QuoteExactParams)>]) -> Vec<Vec<EthereumRpc>> {
    quote_params
        .iter()
        .map(|quote_array| {
            quote_array
                .iter()
                .map(|x| build_quote_exact_request(v4_quoter, &x.1).clone())
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn build_quote_exact_request(v4_quoter: &str, params: &IV4Quoter::QuoteExactParams) -> EthereumRpc {
    let quote = IV4Quoter::quoteExactInputCall { params: params.clone() };
    let call_data: Vec<u8> = quote.abi_encode();
    EthereumRpc::Call(TransactionObject::new_call(v4_quoter, call_data), BlockParameter::Latest)
}

// Returns (amountOut, gasEstimate)
pub fn decode_quoter_response(response: &JsonRpcResponse<String>) -> Result<(U256, U256), SwapperError> {
    let decoded = hex::decode(&response.result).map_err(|_| SwapperError::NetworkError {
        msg: "Failed to decode hex result".into(),
    })?;
    let quoter_return =
        IV4Quoter::quoteExactInputSingleCall::abi_decode_returns(&decoded, true).map_err(|err| SwapperError::ABIError { msg: err.to_string() })?;

    Ok((quoter_return.amountOut, quoter_return.gasEstimate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swapper::uniswap::v4::path::{build_pool_keys, build_quote_exact_params};
    use alloy_core::sol_types::SolValue;
    use gem_evm::uniswap::{path::get_base_pair, FeeTier};
    use gem_hash::keccak::keccak256;
    use primitives::EVMChain;

    #[test]
    fn test_build_quote_exact_single_request() {
        let token_in = EthereumAddress::parse("0x0000000000000000000000000000000000000000").unwrap();
        let token_out = EthereumAddress::parse("0x078D782b760474a361dDA0AF3839290b0EF57AD6").unwrap(); // USDC
        let fee_tiers = vec![FeeTier::ThreeThousand];

        let v4_quoter = "0x333E3C607B141b18fF6de9f258db6e77fE7491E0";
        let amount_in = 10000000000000000_u128;
        let pool_keys = build_pool_keys(&token_in, &token_out, &fee_tiers);

        assert_eq!(pool_keys.len(), 1);

        let pool_key = &pool_keys[0].1;
        let pool_key_bytes = pool_key.abi_encode();
        let pool_id = keccak256(&pool_key_bytes);

        assert_eq!(
            hex::encode_prefixed(pool_id),
            "0x25939956ef14a098d95051d86c75890cfd623a9eeba055e46d8dd9135980b37c"
        );

        let rpc = build_quote_exact_single_request(&token_in, v4_quoter, amount_in, pool_key);

        if let EthereumRpc::Call(call, _) = rpc {
            assert!(call.data.starts_with("0xaa9d21cb"));
        }
    }

    #[test]
    fn test_build_quote_exact_request() {
        let token_in = EthereumAddress::parse("0x6fd9d7AD17242c41f7131d257212c54A0e816691").unwrap(); // UNI
        let token_out = EthereumAddress::parse("0x350a791Bfc2C21F9Ed5d10980Dad2e2638ffa7f6").unwrap(); // LINK
        let fee_tiers = vec![FeeTier::ThreeThousand, FeeTier::FiveHundred, FeeTier::Hundred];
        let base_pair = get_base_pair(&EVMChain::Optimism, false).unwrap();

        let v4_quoter = "0x1f3131a13296fb91c90870043742c3cdbff1a8d7";
        let amount_in = 10000000000000000_u128;

        let quote_params = build_quote_exact_params(amount_in, &token_in, &token_out, &fee_tiers, &base_pair.to_array());
        let rpc_calls = build_quote_exact_requests(v4_quoter, &quote_params);

        assert_eq!(rpc_calls.len(), 3); // 3 intermediaries (ETH, USDC, USDT)

        // 3 fee tiers
        rpc_calls.iter().for_each(|call_array| assert_eq!(call_array.len(), 3));
    }
}
