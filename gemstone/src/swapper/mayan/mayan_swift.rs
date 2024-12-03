use crate::{
    network::{jsonrpc_call, AlienProvider},
    swapper::{ApprovalData, ApprovalType},
};
use alloy_core::{
    hex::{decode as HexDecode, encode_prefixed, ToHexExt},
    primitives::{Address, FixedBytes, U256, U8},
    sol_types::{SolCall, SolValue},
};

use gem_evm::{
    address::EthereumAddress,
    erc20::IERC20,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    mayan::{forwarder::IMayanForwarder::PermitParams, swift::swift::IMayanSwift},
};
use primitives::Chain;
use std::{str::FromStr, sync::Arc};
use thiserror::Error;

pub struct MayanSwift {
    address: String,
    provider: Arc<dyn AlienProvider>,
    chain: Chain,
}

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

#[derive(Debug, Clone)]
pub struct KeyStruct {
    pub params: OrderParams,
    pub token_in: [u8; 32],
    pub chain_id: u16,
    pub protocol_bps: u16,
}

impl KeyStruct {
    pub fn abi_encode(&self) -> Vec<u8> {
        let key = IMayanSwift::KeyStruct {
            params: self.params.to_contract_params(),
            tokenIn: self.token_in.into(),
            chainId: self.chain_id,
            protocolBps: self.protocol_bps,
        };
        key.abi_encode()
    }
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
            v: self.v.into(),
            r: self.r.into(),
            s: self.s.into(),
        }
    }
}

impl MayanSwift {
    pub fn new(address: String, provider: Arc<dyn AlienProvider>, chain: Chain) -> Self {
        Self { address, provider, chain }
    }

    pub async fn encode_create_order_with_eth(&self, params: OrderParams, amount: U256) -> Result<Vec<u8>, MayanSwiftError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{future::pending, time::Duration};

    use async_std::future::timeout;
    use async_trait::async_trait;

    use crate::network::{mock::AlienProviderWarp, AlienError, AlienTarget, Data};

    #[derive(Debug)]
    pub struct AlienProviderMock {
        pub response: String,
        pub timeout: Duration,
    }

    #[async_trait]
    impl AlienProvider for AlienProviderMock {
        async fn request(&self, _target: AlienTarget) -> Result<Data, AlienError> {
            let responses = self.batch_request(vec![_target]).await;
            responses.map(|responses| responses.first().unwrap().clone())
        }

        async fn batch_request(&self, _targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
            let never = pending::<()>();
            let _ = timeout(self.timeout, never).await;
            Ok(vec![self.response.as_bytes().to_vec()])
        }

        fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
            Ok(String::from("http://localhost:8080"))
        }
    }

    // #[test]
    // fn test_encode_amount_hex() {
    //     let amount = U256::from(100);
    //     let mock_provider = AlienProviderMock {
    //         response: String::from("0x0000000000000000000000000000000000000000000000000000000000000064"),
    //         timeout: Duration::from_millis(100),
    //     };
    //     let encoded = MayanSwiftContract::new("0x1234567890abcdef".into(), Arc::new(mock_provider), Chain::Ethereum).encode_amount_hex(amount);
    //     assert_eq!(encoded, "0000000000000000000000000000000000000000000000000000000000000064");
    // }
}
