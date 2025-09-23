use alloy_primitives::{Bytes, U256, hex::decode as HexDecode};
use alloy_sol_types::SolCall;
use gem_evm::{
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    uniswap::contracts::v3::IQuoterV2,
};

use crate::{
    network::JsonRpcResponse,
    swapper::{SwapperError, SwapperMode},
};

pub fn build_quoter_request(mode: &SwapperMode, wallet_address: &str, quoter_v2: &str, amount_in: U256, path: &Bytes) -> EthereumRpc {
    let call_data: Vec<u8> = match mode {
        SwapperMode::ExactIn => IQuoterV2::quoteExactInputCall {
            path: path.clone(),
            amountIn: amount_in,
        }
        .abi_encode(),
        SwapperMode::ExactOut => IQuoterV2::quoteExactOutputCall {
            path: path.clone(),
            amountOut: amount_in,
        }
        .abi_encode(),
    };

    EthereumRpc::Call(
        TransactionObject::new_call_with_from(wallet_address, quoter_v2, call_data),
        BlockParameter::Latest,
    )
}

// Returns (amountOut, gasEstimate)
pub fn decode_quoter_response(response: &JsonRpcResponse<String>) -> Result<(U256, U256), SwapperError> {
    let decoded = HexDecode(&response.result).map_err(|_| SwapperError::NetworkError("Failed to decode quoter response".into()))?;
    let quoter_return = IQuoterV2::quoteExactInputCall::abi_decode_returns(&decoded).map_err(SwapperError::from)?;

    Ok((quoter_return.amountOut, quoter_return.gasEstimate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex::decode as HexDecode;
    use gem_evm::uniswap::contracts::v3::IQuoterV2;

    #[test]
    fn test_decode_quoter_v2_response() {
        let result = "0x0000000000000000000000000000000000000000000000000000000001884eee000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000014b1e00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000004d04db53840b0aec247bb9bd3ffc00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001";
        let decoded = HexDecode(result).unwrap();
        let quote = IQuoterV2::quoteExactInputCall::abi_decode_returns(&decoded).unwrap();

        assert_eq!(quote.amountOut, U256::from(25710318));
        assert_eq!(quote.gasEstimate, U256::from(84766));
    }
}
