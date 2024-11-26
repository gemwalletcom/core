use std::{str::FromStr, sync::Arc};

use alloy_core::{
    hex::decode as HexDecode,
    primitives::{Address, FixedBytes, U256, U8},
    sol_types::SolCall,
};
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    mayan::swift::fee_manager::IFeeManager,
};
use primitives::Chain;
use thiserror::Error;

use crate::network::{jsonrpc_call, AlienProvider};

#[derive(Debug, Error)]
pub enum FeeManagerError {
    #[error("Only operator")]
    OnlyOperator,

    #[error("Only next operator")]
    OnlyNextOperator,

    #[error("Zero address")]
    ZeroAddress,

    #[error("Call failed: {msg}")]
    CallFailed { msg: String },

    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },

    #[error("ABI error: {msg}")]
    ABIError { msg: String },
}

pub struct CalcProtocolBpsParams {
    pub amount_in: u64,
    pub token_in: EthereumAddress,
    pub token_out: FixedBytes<32>, // bytes32
    pub dest_chain: u16,
    pub referrer_bps: u8,
}

pub struct SweepParams {
    pub token: Option<EthereumAddress>, // None for ETH, Some(address) for ERC20
    pub amount: U256,
    pub to: EthereumAddress,
}

#[derive(Debug)]
pub struct FeeManager {
    address: String,
}

impl FeeManager {
    pub fn new(address: String) -> Self {
        Self { address }
    }

    pub async fn calc_protocol_bps(
        &self,
        sender: String,
        chain: &Chain,
        provider: Arc<dyn AlienProvider>,
        params: CalcProtocolBpsParams,
    ) -> Result<U8, FeeManagerError> {
        let token_in_address = Address::from_str(&params.token_in.to_checksum()).map_err(|_| FeeManagerError::InvalidAddress {
            address: params.token_in.to_checksum(),
        })?;

        let call_data = IFeeManager::calcProtocolBpsCall {
            amountIn: params.amount_in,
            tokenIn: token_in_address,
            tokenOut: params.token_out,
            destChain: params.dest_chain,
            referrerBps: params.referrer_bps,
        }
        .abi_encode();

        let calc_protocol_bps_call = EthereumRpc::Call(TransactionObject::new_call_with_from(&sender, &self.address, call_data), BlockParameter::Latest);

        let response = jsonrpc_call(&calc_protocol_bps_call, provider, chain)
            .await
            .map_err(|e| FeeManagerError::CallFailed { msg: e.to_string() })?;

        let result: String = response.extract_result().map_err(|e| FeeManagerError::CallFailed { msg: e.to_string() })?;

        let decoded = HexDecode(&result).map_err(|e| FeeManagerError::ABIError {
            msg: format!("Failed to decode hex result: {}", e),
        })?;

        let calculated_bps = IFeeManager::calcProtocolBpsCall::abi_decode_returns(&decoded, false).map_err(|e| FeeManagerError::ABIError {
            msg: format!("Invalid calcProtocolBpsCall response: {}", e),
        })?;

        Ok(U8::from(calculated_bps._0))
    }
}
