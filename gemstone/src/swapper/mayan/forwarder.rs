use std::{str::FromStr, sync::Arc};

use alloy_core::{
    primitives::{Address, U256},
    sol_types::{SolCall, SolValue},
};
use gem_evm::{
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    mayan::forwarder::IMayanForwarder,
};
use primitives::Chain;
use thiserror::Error;

use crate::network::{jsonrpc_call, AlienProvider};

#[derive(Debug, Error)]
pub enum MayanForwarderError {
    #[error("Unsupported protocol")]
    UnsupportedProtocol,
    #[error("Call failed: {msg}")]
    CallFailed { msg: String },
    #[error("Invalid response")]
    InvalidResponse,
    #[error("ABI error: {msg}")]
    ABIError { msg: String },
}

pub struct MayanForwarder {
    address: String,
    provider: Arc<dyn AlienProvider>,
    chain: Chain,
}

impl MayanForwarder {
    pub fn new(address: String, provider: Arc<dyn AlienProvider>, chain: Chain) -> Self {
        Self { address, provider, chain }
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
}
