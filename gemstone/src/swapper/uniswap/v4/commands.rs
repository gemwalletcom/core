use crate::swapper::{SwapQuoteRequest, SwapperError};
use alloy_primitives::U256;
use gem_evm::{
    address::EthereumAddress,
    uniswap::command::{Permit2Permit, UniversalRouterCommand},
};

#[allow(unused)]
pub fn build_commands(
    request: &SwapQuoteRequest,
    token_in: &EthereumAddress,
    token_out: &EthereumAddress,
    amount_in: U256,
    quote_amount: U256,
    permit: Option<Permit2Permit>,
    fee_token_is_input: bool,
) -> Result<Vec<UniversalRouterCommand>, SwapperError> {
    todo!()
}

#[allow(unused)]
pub fn encode_commands(commands: &[UniversalRouterCommand], deadline: U256) -> Vec<u8> {
    todo!()
}
