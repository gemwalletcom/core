use std::str::FromStr;

use alloy_core::{
    primitives::{Address, U256},
    sol_types::SolCall,
};
use gem_evm::mayan::forwarder::IMayanForwarder;
use thiserror::Error;

use super::swift::MayanSwiftPermit;

#[derive(Debug, Error)]
pub enum MayanForwarderError {
    #[error("ABI Error: {msg}")]
    ABIError { msg: String },
}

pub struct MayanForwarder {}

impl MayanForwarder {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn encode_forward_eth_call(&self, mayan_protocol: &str, protocol_data: Vec<u8>) -> Result<Vec<u8>, MayanForwarderError> {
        let call_data = IMayanForwarder::forwardEthCall {
            mayanProtocol: Address::from_str(mayan_protocol).map_err(|e| MayanForwarderError::ABIError {
                msg: format!("Invalid protocol address: {}", e),
            })?,
            protocolData: protocol_data.into(),
        }
        .abi_encode();

        Ok(call_data)
    }

    pub fn encode_forward_erc20_call(
        &self,
        token_in: &str,
        amount_in: U256,
        permit: Option<MayanSwiftPermit>,
        mayan_protocol: &str,
        protocol_data: Vec<u8>,
    ) -> Result<Vec<u8>, MayanForwarderError> {
        let call_data = IMayanForwarder::forwardERC20Call {
            tokenIn: Address::from_str(token_in).map_err(|e| MayanForwarderError::ABIError {
                msg: format!("Invalid token address: {}", e),
            })?,
            amountIn: amount_in,
            permitParams: permit.map_or(MayanSwiftPermit::zero().to_contract_params(), |p| p.to_contract_params()),
            mayanProtocol: Address::from_str(mayan_protocol).map_err(|e| MayanForwarderError::ABIError {
                msg: format!("Invalid protocol address: {}", e),
            })?,
            protocolData: protocol_data.into(),
        }
        .abi_encode();

        Ok(call_data)
    }

    pub async fn encode_swap_and_forward_eth_call(
        &self,
        amount_in: U256,
        swap_protocol: &str,
        swap_data: Vec<u8>,
        middle_token: &str,
        min_middle_amount: U256,
        mayan_protocol: &str,
        mayan_data: Vec<u8>,
    ) -> Result<Vec<u8>, MayanForwarderError> {
        let call_data = IMayanForwarder::swapAndForwardEthCall {
            amountIn: amount_in,
            swapProtocol: Address::from_str(swap_protocol).map_err(|e| MayanForwarderError::ABIError {
                msg: format!("Invalid swap protocol address: {}", e),
            })?,
            swapData: swap_data.into(),
            middleToken: Address::from_str(middle_token).map_err(|e| MayanForwarderError::ABIError {
                msg: format!("Invalid middle token address: {}", e),
            })?,
            minMiddleAmount: min_middle_amount,
            mayanProtocol: Address::from_str(mayan_protocol).map_err(|e| MayanForwarderError::ABIError {
                msg: format!("Invalid mayan protocol address: {}", e),
            })?,
            mayanData: mayan_data.into(),
        }
        .abi_encode();

        Ok(call_data)
    }

    pub fn encode_swap_and_forward_erc20_call(
        &self,
        token_in: &str,
        amount_in: U256,
        permit: Option<MayanSwiftPermit>,
        swap_protocol: &str,
        swap_data: Vec<u8>,
        middle_token: &str,
        min_middle_amount: U256,
        mayan_protocol: &str,
        mayan_data: Vec<u8>,
    ) -> Result<Vec<u8>, MayanForwarderError> {
        let call_data = IMayanForwarder::swapAndForwardERC20Call {
            tokenIn: Address::from_str(token_in).map_err(|e| MayanForwarderError::ABIError {
                msg: format!("Invalid token address: {}", e),
            })?,
            amountIn: amount_in,
            permitParams: permit.map_or(MayanSwiftPermit::zero().to_contract_params(), |p| p.to_contract_params()),
            swapProtocol: Address::from_str(swap_protocol).map_err(|e| MayanForwarderError::ABIError {
                msg: format!("Invalid swap protocol address: {}", e),
            })?,
            swapData: swap_data.into(),
            middleToken: Address::from_str(middle_token).map_err(|e| MayanForwarderError::ABIError {
                msg: format!("Invalid middle token address: {}", e),
            })?,
            minMiddleAmount: min_middle_amount,
            mayanProtocol: Address::from_str(mayan_protocol).map_err(|e| MayanForwarderError::ABIError {
                msg: format!("Invalid mayan protocol address: {}", e),
            })?,
            mayanData: mayan_data.into(),
        }
        .abi_encode();

        Ok(call_data)
    }
}
