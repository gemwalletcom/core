use alloy_core::{
    primitives::{Address, U256},
    sol_types::SolCall,
};

use super::swift::MayanSwiftPermit;
use crate::swapper::SwapperError;
use gem_evm::mayan::contracts::IMayanForwarder;

#[derive(Default)]
pub struct MayanForwarder {}

impl MayanForwarder {
    pub fn encode_forward_eth_call(&self, mayan_protocol: Address, protocol_data: Vec<u8>) -> Result<Vec<u8>, SwapperError> {
        let call_data = IMayanForwarder::forwardEthCall {
            mayanProtocol: mayan_protocol,
            protocolData: protocol_data.into(),
        }
        .abi_encode();

        Ok(call_data)
    }

    pub fn encode_forward_erc20_call(
        &self,
        token_in: Address,
        amount_in: U256,
        permit: Option<MayanSwiftPermit>,
        mayan_protocol: Address,
        protocol_data: Vec<u8>,
    ) -> Result<Vec<u8>, SwapperError> {
        let call_data = IMayanForwarder::forwardERC20Call {
            tokenIn: token_in,
            amountIn: amount_in,
            permitParams: permit.map_or(MayanSwiftPermit::zero().to_contract_params(), |p| p.to_contract_params()),
            mayanProtocol: mayan_protocol,
            protocolData: protocol_data.into(),
        }
        .abi_encode();

        Ok(call_data)
    }

    pub fn encode_swap_and_forward_eth_call(
        &self,
        amount_in: U256,
        swap_protocol: Address,
        swap_data: Vec<u8>,
        middle_token: Address,
        min_middle_amount: U256,
        mayan_protocol: Address,
        mayan_data: Vec<u8>,
    ) -> Result<Vec<u8>, SwapperError> {
        let call_data = IMayanForwarder::swapAndForwardEthCall {
            amountIn: amount_in,
            swapProtocol: swap_protocol,
            swapData: swap_data.into(),
            middleToken: middle_token,
            minMiddleAmount: min_middle_amount,
            mayanProtocol: mayan_protocol,
            mayanData: mayan_data.into(),
        }
        .abi_encode();

        Ok(call_data)
    }

    pub fn encode_swap_and_forward_erc20_call(
        &self,
        token_in: Address,
        amount_in: U256,
        permit: Option<MayanSwiftPermit>,
        swap_protocol: Address,
        swap_data: Vec<u8>,
        middle_token: Address,
        min_middle_amount: U256,
        mayan_protocol: Address,
        mayan_data: Vec<u8>,
    ) -> Result<Vec<u8>, SwapperError> {
        let call_data = IMayanForwarder::swapAndForwardERC20Call {
            tokenIn: token_in,
            amountIn: amount_in,
            permitParams: permit.map_or(MayanSwiftPermit::zero().to_contract_params(), |p| p.to_contract_params()),
            swapProtocol: swap_protocol,
            swapData: swap_data.into(),
            middleToken: middle_token,
            minMiddleAmount: min_middle_amount,
            mayanProtocol: mayan_protocol,
            mayanData: mayan_data.into(),
        }
        .abi_encode();

        Ok(call_data)
    }
}
