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
    multicall3::IMulticall3,
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
    pub fn paused_call3(&self) -> IMulticall3::Call3 {
        IMulticall3::Call3 {
            target: self.contract.parse().unwrap(),
            allowFailure: true,
            callData: HubPoolInterface::pausedCall {}.abi_encode().into(),
        }
    }

    pub fn decoded_paused_call3(&self, result: &IMulticall3::Result) -> Result<bool, SwapperError> {
        if result.success {
            let decoded =
                HubPoolInterface::pausedCall::abi_decode_returns(&result.returnData, true).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;
            Ok(decoded._0)
        } else {
            Err(SwapperError::ABIError {
                msg: "paused call failed".into(),
            })
        }
    }

    pub fn sync_call3(&self, l1token: &EthereumAddress) -> IMulticall3::Call3 {
        IMulticall3::Call3 {
            target: self.contract.parse().unwrap(),
            allowFailure: true,
            callData: HubPoolInterface::syncCall {
                l1Token: Address::from_slice(&l1token.bytes),
            }
            .abi_encode()
            .into(),
        }
    }

    pub fn pooled_token_call3(&self, l1token: &EthereumAddress) -> IMulticall3::Call3 {
        IMulticall3::Call3 {
            target: self.contract.parse().unwrap(),
            allowFailure: true,
            callData: HubPoolInterface::pooledTokensCall {
                l1Token: Address::from_slice(&l1token.bytes),
            }
            .abi_encode()
            .into(),
        }
    }

    pub fn decoded_pooled_token_call3(&self, result: &IMulticall3::Result) -> Result<HubPoolInterface::PooledToken, SwapperError> {
        if result.success {
            let decoded =
                HubPoolInterface::pooledTokensCall::abi_decode_returns(&result.returnData, true).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;
            Ok(decoded._0)
        } else {
            Err(SwapperError::ABIError {
                msg: "pooled token call failed".into(),
            })
        }
    }

    pub fn utilization_call3(&self, l1_token: &EthereumAddress, amount: U256) -> IMulticall3::Call3 {
        let l1_token = Address::from_slice(&l1_token.bytes);
        let data = if amount.is_zero() {
            HubPoolInterface::liquidityUtilizationCurrentCall { l1Token: l1_token }.abi_encode()
        } else {
            HubPoolInterface::liquidityUtilizationPostRelayCall {
                l1Token: l1_token,
                relayedAmount: amount,
            }
            .abi_encode()
        };
        IMulticall3::Call3 {
            target: self.contract.parse().unwrap(),
            allowFailure: true,
            callData: data.into(),
        }
    }

    pub fn decoded_utilization_call3(&self, result: &IMulticall3::Result) -> Result<BigInt, SwapperError> {
        if result.success {
            let value = HubPoolInterface::liquidityUtilizationCurrentCall::abi_decode_returns(&result.returnData, true)
                .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
                ._0;
            Ok(BigInt::from_bytes_le(num_bigint::Sign::Plus, &value.to_le_bytes::<32>()))
        } else {
            Err(SwapperError::ABIError {
                msg: "utilization call failed".into(),
            })
        }
    }

    pub async fn fetch_utilization(&self, pool_token: &EthereumAddress, amount: U256) -> Result<BigInt, SwapperError> {
        let call3 = self.utilization_call3(pool_token, amount);
        let call = EthereumRpc::Call(TransactionObject::new_call(&self.contract, call3.callData.to_vec()), BlockParameter::Latest);
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
