use crate::{network::JsonRpcResponse, swapper::SwapperError};
use alloy_core::{
    hex,
    primitives::Address,
    sol_types::{SolCall, SolValue},
};
use alloy_primitives::{Bytes, U256};
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    uniswap::contracts::v4::{IV4Quoter, PathKey, PoolKey},
};

pub fn build_quote_exact_single_request(token_in: &EthereumAddress, wallet_address: &str, quoter_v2: &str, amount_in: u128, pool: &PoolKey) -> EthereumRpc {
    let token_in_address = Address::from_slice(&token_in.bytes);
    let zero_for_one = token_in_address == pool.currency0;
    let quote_single = IV4Quoter::QuoteExactSingleParams {
        poolKey: pool.clone(),
        zeroForOne: zero_for_one,
        exactAmount: amount_in,
        hookData: Bytes::new(),
    };
    let call_data: Vec<u8> = quote_single.abi_encode();
    EthereumRpc::Call(
        TransactionObject::new_call_with_from(wallet_address, quoter_v2, call_data),
        BlockParameter::Latest,
    )
}

pub fn build_quote_exact_request(token_in: &EthereumAddress, wallet_address: &str, quoter_v2: &str, amount_in: u128, path: &[PathKey]) -> EthereumRpc {
    let quote = IV4Quoter::QuoteExactParams {
        exactCurrency: Address::from_slice(&token_in.bytes),
        path: path.to_vec(),
        exactAmount: amount_in,
    };
    let call_data: Vec<u8> = quote.abi_encode();
    EthereumRpc::Call(
        TransactionObject::new_call_with_from(wallet_address, quoter_v2, call_data),
        BlockParameter::Latest,
    )
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
