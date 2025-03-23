use alloy_core::{
    primitives::{Address, U256},
    sol_types::SolCall,
};

use gem_evm::mayan::{contracts::IMayanForwarder::PermitParams, contracts::IMayanSwift};
use std::str::FromStr;
use thiserror::Error;

#[derive(Default)]
pub struct MayanSwift {}

#[derive(Error, Debug)]
pub enum MayanSwiftError {
    #[error("Call failed: {msg}")]
    CallFailed { msg: String },

    #[error("Invalid response: {msg}")]
    InvalidResponse { msg: String },

    #[error("ABI error: {msg}")]
    ABIError { msg: String },

    #[error("Invalid amount")]
    InvalidAmount,
}

// Parameter structs with native types
#[derive(Debug, Clone)]
pub struct OrderParams {
    pub trader: [u8; 32],
    pub token_out: [u8; 32],
    pub min_amount_out: u64,
    pub gas_drop: u64,
    pub cancel_fee: u64,
    pub refund_fee: u64,
    pub deadline: u64,
    pub dest_addr: [u8; 32],
    pub dest_chain_id: u16,
    pub referrer_addr: [u8; 32],
    pub referrer_bps: u8,
    pub auction_mode: u8,
    pub random: [u8; 32],
}

impl OrderParams {
    pub fn to_contract_params(&self) -> IMayanSwift::OrderParams {
        IMayanSwift::OrderParams {
            trader: self.trader.into(),
            tokenOut: self.token_out.into(),
            minAmountOut: self.min_amount_out,
            gasDrop: self.gas_drop,
            cancelFee: self.cancel_fee,
            refundFee: self.refund_fee,
            deadline: self.deadline,
            destAddr: self.dest_addr.into(),
            destChainId: self.dest_chain_id,
            referrerAddr: self.referrer_addr.into(),
            referrerBps: self.referrer_bps,
            auctionMode: self.auction_mode,
            random: self.random.into(),
        }
    }
}

#[derive(Debug)]
pub struct MayanSwiftPermit {
    pub value: String,
    pub deadline: u64,
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
}

impl MayanSwiftPermit {
    pub fn zero() -> Self {
        Self {
            value: "0".to_string(),
            deadline: 0,
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
        }
    }

    pub fn to_contract_params(&self) -> PermitParams {
        PermitParams {
            value: U256::from_str(&self.value).unwrap(),
            deadline: U256::from(self.deadline),
            v: self.v,
            r: self.r.into(),
            s: self.s.into(),
        }
    }
}

impl MayanSwift {
    pub async fn encode_create_order_with_eth(&self, params: OrderParams) -> Result<Vec<u8>, MayanSwiftError> {
        let call_data = IMayanSwift::createOrderWithEthCall {
            params: params.to_contract_params(),
        }
        .abi_encode();

        Ok(call_data)
    }

    pub async fn encode_create_order_with_token(&self, token_in: &str, amount: U256, params: OrderParams) -> Result<Vec<u8>, MayanSwiftError> {
        let call_data = IMayanSwift::createOrderWithTokenCall {
            tokenIn: Address::from_str(token_in).map_err(|e| MayanSwiftError::ABIError {
                msg: format!("Invalid token address: {}", e),
            })?,
            amountIn: amount,
            params: params.to_contract_params(),
        }
        .abi_encode();

        Ok(call_data)
    }
}
