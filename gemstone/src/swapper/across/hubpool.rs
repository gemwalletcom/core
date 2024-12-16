use crate::{
    network::{
        jsonrpc::{jsonrpc_call, JsonRpcResult},
        AlienProvider,
    },
    swapper::SwapperError,
};

use alloy_core::sol_types::SolCall;
use alloy_primitives::{hex::decode as HexDecode, Address, U256};
use gem_evm::{
    across::contracts::HubPoolInterface,
    address::EthereumAddress,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
};
use num_bigint::BigInt;
use primitives::Chain;
use std::sync::Arc;

pub struct HubPoolClient {
    pub contract: String,
    pub provider: Arc<dyn AlienProvider>,
    pub chain: Chain,
}

impl HubPoolClient {
    pub async fn is_paused(&self) -> Result<bool, SwapperError> {
        let data = HubPoolInterface::pausedCall {}.abi_encode();
        let call = EthereumRpc::Call(TransactionObject::new_call(&self.contract, data), BlockParameter::Latest);
        let response: JsonRpcResult<String> = jsonrpc_call(&call, self.provider.clone(), &self.chain).await?;
        let result = response.take()?;
        let hex_data = HexDecode(result).map_err(|_| SwapperError::InvalidAmount)?;
        let result = HubPoolInterface::pausedCall::abi_decode_returns(&hex_data, true)
            .map_err(|_| SwapperError::InvalidAmount)?
            ._0;
        Ok(result)
    }

    pub async fn fetch_utilization(&self, pool_token: &EthereumAddress, amount: U256) -> Result<BigInt, SwapperError> {
        let l1_token = Address::from_slice(&pool_token.bytes);
        let data = if amount.is_zero() {
            HubPoolInterface::liquidityUtilizationCurrentCall { l1Token: l1_token }.abi_encode()
        } else {
            HubPoolInterface::liquidityUtilizationPostRelayCall {
                l1Token: l1_token,
                relayedAmount: amount,
            }
            .abi_encode()
        };
        let call = EthereumRpc::Call(TransactionObject::new_call(&self.contract, data), BlockParameter::Latest);
        let response: JsonRpcResult<String> = jsonrpc_call(&call, self.provider.clone(), &self.chain).await?;
        let result = response.take()?;
        let hex_data = HexDecode(result).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        let value = HubPoolInterface::liquidityUtilizationCurrentCall::abi_decode_returns(&hex_data, true)
            .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
            ._0;
        let result = BigInt::from_bytes_le(num_bigint::Sign::Plus, &value.to_le_bytes::<32>());
        Ok(result)
    }
}
