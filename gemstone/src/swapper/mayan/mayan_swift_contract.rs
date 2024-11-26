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
    mayan::swift::swift::MayanSwift,
};
use primitives::Chain;
use std::{str::FromStr, sync::Arc};
use thiserror::Error;

pub struct MayanSwiftContract {
    address: String,
    provider: Arc<dyn AlienProvider>,
    chain: Chain,
}

#[derive(Error, Debug)]
pub enum MayanSwiftContractError {
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

#[derive(Debug)]
pub struct PermitParams {
    pub value: String,
    pub deadline: u64,
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
}

impl MayanSwiftContract {
    pub fn new(address: String, provider: Arc<dyn AlienProvider>, chain: Chain) -> Self {
        Self { address, provider, chain }
    }

    pub async fn get_fee_manager_address(&self) -> Result<String, MayanSwiftContractError> {
        let call_data = MayanSwift::feeManagerCall {}.abi_encode();
        let fee_manager_call = EthereumRpc::Call(TransactionObject::new_call(&self.address, call_data), BlockParameter::Latest);

        let response = jsonrpc_call(&fee_manager_call, self.provider.clone(), &self.chain)
            .await
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let result: String = response
            .extract_result()
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let decoded = HexDecode(&result).map_err(|e| MayanSwiftContractError::InvalidResponse { msg: e.to_string() })?;

        let fee_manager =
            MayanSwift::feeManagerCall::abi_decode_returns(&decoded, false).map_err(|e| MayanSwiftContractError::ABIError { msg: e.to_string() })?;

        let address = EthereumAddress::from_str(&fee_manager.feeManager.to_string()).map_err(|e| MayanSwiftContractError::ABIError {
            msg: format!("Failed to parse fee manager address: {}", e),
        })?;

        Ok(address.to_checksum())
    }

    pub async fn create_order_with_eth(&self, from: &str, params: OrderParams, value: &str) -> Result<String, MayanSwiftContractError> {
        let call_data = self
            .encode_create_order_with_eth(params, U256::from_str(value).map_err(|_| MayanSwiftContractError::InvalidAmount)?)
            .await?;

        let create_order_call = EthereumRpc::Call(
            TransactionObject::new_call_with_value(from, &self.address, call_data, value),
            BlockParameter::Latest,
        );

        let response = jsonrpc_call(&create_order_call, self.provider.clone(), &self.chain)
            .await
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let result: String = response
            .extract_result()
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        Ok(result)
    }

    pub async fn create_order_with_token(&self, from: &str, token_in: &str, amount_in: &str, params: OrderParams) -> Result<String, MayanSwiftContractError> {
        let call_data = self
            .encode_create_order_with_token(token_in, U256::from_str(amount_in).map_err(|_| MayanSwiftContractError::InvalidAmount)?, params)
            .await?;

        let create_order_call = EthereumRpc::Call(TransactionObject::new_call_with_from(from, &self.address, call_data), BlockParameter::Latest);

        let response = jsonrpc_call(&create_order_call, self.provider.clone(), &self.chain)
            .await
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let result: String = response
            .extract_result()
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        Ok(result)
    }

    pub async fn estimate_create_order_with_eth(&self, from: &str, params: OrderParams, amount: U256) -> Result<U256, MayanSwiftContractError> {
        let call_data = self.encode_create_order_with_eth(params, amount).await?;

        // let value = encode_prefixed(amount.to_be_bytes_vec());
        let value = format!("0x{:x}", amount);

        let estimate_gas_call = EthereumRpc::EstimateGas(TransactionObject::new_call_with_value(from, &self.address, call_data, &value));

        let response = jsonrpc_call(&estimate_gas_call, self.provider.clone(), &self.chain)
            .await
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let result: String = response
            .extract_result()
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let hex_str = result.trim_start_matches("0x");

        Ok(U256::from_str_radix(hex_str, 16).map_err(|e| MayanSwiftContractError::InvalidResponse { msg: e.to_string() })?)
    }

    pub async fn estimate_create_order_with_token(&self, token_in: &str, amount: U256, params: OrderParams) -> Result<U256, MayanSwiftContractError> {
        let call_data = self.encode_create_order_with_token(token_in, amount, params).await?;
        let estimate_gas_call = EthereumRpc::EstimateGas(TransactionObject::new_call_with_value(&self.address, token_in, call_data, &amount.to_string()));

        let response = jsonrpc_call(&estimate_gas_call, self.provider.clone(), &self.chain)
            .await
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let result: String = response
            .extract_result()
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let decoded = HexDecode(&result).map_err(|e| MayanSwiftContractError::InvalidResponse { msg: e.to_string() })?;

        Ok(U256::from_str(decoded.encode_hex().as_str()).map_err(|e| MayanSwiftContractError::InvalidResponse { msg: e.to_string() })?)
    }

    pub async fn encode_create_order_with_eth(&self, params: OrderParams, amount: U256) -> Result<Vec<u8>, MayanSwiftContractError> {
        let call_data = MayanSwift::createOrderWithEthCall {
            params: self.convert_order_params(params),
        }
        .abi_encode();

        Ok(call_data)
    }

    pub async fn encode_create_order_with_token(&self, token_in: &str, amount: U256, params: OrderParams) -> Result<Vec<u8>, MayanSwiftContractError> {
        let call_data = MayanSwift::createOrderWithTokenCall {
            tokenIn: Address::from_str(token_in).map_err(|e| MayanSwiftContractError::ABIError {
                msg: format!("Invalid token address: {}", e),
            })?,
            amountIn: amount,
            params: self.convert_order_params(params),
        }
        .abi_encode();

        Ok(call_data)
    }

    pub async fn get_orders(&self, order_hashes: Vec<[u8; 32]>) -> Result<Vec<(u8, u64, u16)>, MayanSwiftContractError> {
        let call_data = MayanSwift::getOrdersCall {
            orderHashes: order_hashes.into_iter().map(|x| x.into()).collect(),
        }
        .abi_encode();

        let get_orders_call = EthereumRpc::Call(TransactionObject::new_call(&self.address, call_data), BlockParameter::Latest);

        let response = jsonrpc_call(&get_orders_call, self.provider.clone(), &self.chain)
            .await
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let result: String = response
            .extract_result()
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let decoded = HexDecode(&result).map_err(|e| MayanSwiftContractError::InvalidResponse { msg: e.to_string() })?;

        let orders = MayanSwift::getOrdersCall::abi_decode_returns(&decoded, false).map_err(|e| MayanSwiftContractError::ABIError { msg: e.to_string() })?;

        Ok(orders
            ._0
            .into_iter()
            .map(|order| (order.status, order.amountIn.try_into().unwrap_or(0), order.destChainId))
            .collect())
    }

    pub async fn check_token_approval(&self, owner: &str, token: &str, amount: &str) -> Result<ApprovalType, MayanSwiftContractError> {
        // Encode allowance call for ERC20 token
        let call_data = IERC20::allowanceCall {
            owner: Address::from_str(owner).map_err(|e| MayanSwiftContractError::ABIError {
                msg: format!("Invalid owner address: {}", e),
            })?,
            spender: Address::from_str(&self.address).map_err(|e| MayanSwiftContractError::ABIError {
                msg: format!("Invalid spender address: {}", e),
            })?,
        }
        .abi_encode();

        // Create RPC call
        let allowance_call = EthereumRpc::Call(TransactionObject::new_call(token, call_data), BlockParameter::Latest);

        // Execute the call
        let response = jsonrpc_call(&allowance_call, self.provider.clone(), &self.chain)
            .await
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let result: String = response
            .extract_result()
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        // Decode the response
        let decoded = hex::decode(result.trim_start_matches("0x")).map_err(|e| MayanSwiftContractError::InvalidResponse { msg: e.to_string() })?;

        let allowance = IERC20::allowanceCall::abi_decode_returns(&decoded, false).map_err(|e| MayanSwiftContractError::ABIError { msg: e.to_string() })?;

        // Convert amount string to U256 for comparison
        let required_amount = U256::from_str(amount).map_err(|e| MayanSwiftContractError::ABIError {
            msg: format!("Invalid amount: {}", e),
        })?;

        // Compare allowance with required amount
        Ok(if allowance._0 >= required_amount {
            ApprovalType::Approve(ApprovalData {
                token: token.into(),
                spender: self.address.clone(),
                value: amount.into(),
            })
        } else {
            ApprovalType::None
        })
    }

    pub async fn encode_create_order_with_sig(
        &self,
        token_in: &str,
        amount_in: U256,
        params: OrderParams,
        submission_fee: U256,
        signed_order_hash: Vec<u8>,
        permit_params: PermitParams,
    ) -> Result<Vec<u8>, MayanSwiftContractError> {
        let call_data = MayanSwift::createOrderWithSigCall {
            tokenIn: Address::from_str(token_in).map_err(|e| MayanSwiftContractError::ABIError {
                msg: format!("Invalid token address: {}", e),
            })?,
            amountIn: amount_in,
            params: self.convert_order_params(params),
            submissionFee: submission_fee,
            signedOrderHash: signed_order_hash.into(),
            permitParams: self.convert_permit_params(permit_params),
        }
        .abi_encode();

        Ok(call_data)
    }

    pub async fn create_order_with_sig(
        &self,
        from: &str,
        token_in: &str,
        amount_in: &str,
        params: OrderParams,
        submission_fee: &str,
        signed_order_hash: Vec<u8>,
        permit_params: PermitParams,
    ) -> Result<String, MayanSwiftContractError> {
        let call_data = self
            .encode_create_order_with_sig(
                token_in,
                U256::from_str(amount_in).map_err(|_| MayanSwiftContractError::InvalidAmount)?,
                params,
                U256::from_str(submission_fee).map_err(|_| MayanSwiftContractError::InvalidAmount)?,
                signed_order_hash,
                permit_params,
            )
            .await?;

        let create_order_call = EthereumRpc::Call(TransactionObject::new_call_with_from(from, &self.address, call_data), BlockParameter::Latest);

        let response = jsonrpc_call(&create_order_call, self.provider.clone(), &self.chain)
            .await
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        let result: String = response
            .extract_result()
            .map_err(|e| MayanSwiftContractError::CallFailed { msg: e.to_string() })?;

        Ok(result)
    }

    pub fn convert_permit_params(&self, permit_params: PermitParams) -> MayanSwift::PermitParams {
        MayanSwift::PermitParams {
            value: U256::from_str(&permit_params.value).map_err(|_| MayanSwiftContractError::InvalidAmount)?,
            deadline: U256::from(permit_params.deadline),
            v: permit_params.v.into(),
            r: permit_params.r.into(),
            s: permit_params.s.into(),
        }
    }

    // Helper method to convert our native OrderParams to contract format
    pub fn convert_order_params(&self, params: OrderParams) -> MayanSwift::OrderParams {
        MayanSwift::OrderParams {
            trader: params.trader.into(),
            tokenOut: params.token_out.into(),
            minAmountOut: params.min_amount_out,
            gasDrop: params.gas_drop,
            cancelFee: params.cancel_fee,
            refundFee: params.refund_fee,
            deadline: params.deadline,
            destAddr: params.dest_addr.into(),
            destChainId: params.dest_chain_id,
            referrerAddr: params.referrer_addr.into(),
            referrerBps: params.referrer_bps,
            auctionMode: params.auction_mode,
            random: params.random.into(),
        }
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
