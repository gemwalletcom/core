use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use alloy_primitives::{Address, U256, hex::decode as HexDecode};
use alloy_sol_types::SolCall;
use gem_client::Client;
use num_bigint::{BigInt, Sign};
use primitives::Chain;

use crate::{
    network::{EvmRpcClientFactory},
    swapper::SwapperError,
};
use gem_evm::{
    across::{contracts::HubPoolInterface, deployment::ACROSS_HUBPOOL},
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    multicall3::{IMulticall3, create_call3, decode_call3_return},
};

pub struct HubPoolClient<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    pub contract: String,
    rpc_factory: Arc<F>,
    pub chain: Chain,
    _phantom: PhantomData<C>,
}

impl<C, F> HubPoolClient<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    pub fn new(rpc_factory: Arc<F>, chain: Chain) -> Self {
        Self {
            contract: ACROSS_HUBPOOL.into(),
            rpc_factory,
            chain,
            _phantom: PhantomData,
        }
    }

    pub fn paused_call3(&self) -> IMulticall3::Call3 {
        create_call3(&self.contract, HubPoolInterface::pausedCall {})
    }

    pub fn decoded_paused_call3(&self, result: &IMulticall3::Result) -> Result<bool, SwapperError> {
        let value = decode_call3_return::<HubPoolInterface::pausedCall>(result).map_err(|e| SwapperError::ABIError(e.to_string()))?;
        Ok(value)
    }

    pub fn sync_call3(&self, l1token: &Address) -> IMulticall3::Call3 {
        IMulticall3::Call3 {
            target: self.contract.parse().unwrap(),
            allowFailure: true,
            callData: HubPoolInterface::syncCall { l1Token: *l1token }.abi_encode().into(),
        }
    }

    pub fn pooled_token_call3(&self, l1token: &Address) -> IMulticall3::Call3 {
        IMulticall3::Call3 {
            target: self.contract.parse().unwrap(),
            allowFailure: true,
            callData: HubPoolInterface::pooledTokensCall { l1Token: *l1token }.abi_encode().into(),
        }
    }

    pub fn decoded_pooled_token_call3(&self, result: &IMulticall3::Result) -> Result<HubPoolInterface::PooledToken, SwapperError> {
        if result.success {
            let decoded = HubPoolInterface::pooledTokensCall::abi_decode_returns(&result.returnData).map_err(|e| SwapperError::ABIError(e.to_string()))?;
            Ok(decoded)
        } else {
            Err(SwapperError::ABIError("pooled token call failed".into()))
        }
    }

    pub fn utilization_call3(&self, l1_token: &Address, amount: U256) -> IMulticall3::Call3 {
        let data = if amount.is_zero() {
            HubPoolInterface::liquidityUtilizationCurrentCall { l1Token: *l1_token }.abi_encode()
        } else {
            HubPoolInterface::liquidityUtilizationPostRelayCall {
                l1Token: *l1_token,
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
            let value = HubPoolInterface::liquidityUtilizationCurrentCall::abi_decode_returns(&result.returnData).map_err(SwapperError::from)?;

            Ok(BigInt::from_bytes_le(Sign::Plus, &value.to_le_bytes::<32>()))
        } else {
            Err(SwapperError::ABIError("utilization call failed".into()))
        }
    }

    pub fn get_current_time(&self) -> IMulticall3::Call3 {
        create_call3(&self.contract, HubPoolInterface::getCurrentTimeCall {})
    }

    pub fn decoded_current_time(&self, result: &IMulticall3::Result) -> Result<u32, SwapperError> {
        let value = decode_call3_return::<HubPoolInterface::getCurrentTimeCall>(result).map_err(|e| SwapperError::ABIError(e.to_string()))?;
        value.try_into().map_err(|_| SwapperError::ABIError("decode current time failed".into()))
    }

    pub async fn fetch_utilization(&self, pool_token: &Address, amount: U256) -> Result<BigInt, SwapperError> {
        let call3 = self.utilization_call3(pool_token, amount);
        let call = EthereumRpc::Call(TransactionObject::new_call(&self.contract, call3.callData.to_vec()), BlockParameter::Latest);
        let client = self.rpc_factory.client_for(self.chain).map_err(SwapperError::from)?;
        let result: String = client.request(call).await?;
        let hex_data = HexDecode(result).map_err(|e| SwapperError::NetworkError(e.to_string()))?;
        let value = HubPoolInterface::liquidityUtilizationCurrentCall::abi_decode_returns(&hex_data).map_err(SwapperError::from)?;
        let result = BigInt::from_bytes_le(num_bigint::Sign::Plus, &value.to_le_bytes::<32>());
        Ok(result)
    }
}
