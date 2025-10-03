use crate::{
    network::{EvmRpcClientFactory, JsonRpcResult},
    swapper::SwapperError,
};
use alloy_primitives::{Address, hex::decode as HexDecode};
use alloy_sol_types::SolCall;
use gem_client::Client;
use gem_evm::{
    across::{contracts::AcrossConfigStore, deployment::ACROSS_CONFIG_STORE, fees},
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    multicall3::IMulticall3,
};
use primitives::Chain;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, marker::PhantomData, sync::Arc};

const CONFIG_CACHE_TTL: u64 = 60 * 60 * 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateModel {
    #[serde(rename = "UBar")]
    pub ubar: String,
    #[serde(rename = "R0")]
    pub r0: String,
    #[serde(rename = "R1")]
    pub r1: String,
    #[serde(rename = "R2")]
    pub r2: String,
}

impl From<RateModel> for fees::RateModel {
    fn from(value: RateModel) -> Self {
        Self {
            ubar: value.ubar.parse().unwrap(),
            r0: value.r0.parse().unwrap(),
            r1: value.r1.parse().unwrap(),
            r2: value.r2.parse().unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenConfig {
    pub rate_model: RateModel,
    pub route_rate_model: HashMap<String, RateModel>,
}

pub struct ConfigStoreClient<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    pub contract: String,
    rpc_factory: Arc<F>,
    chain: Chain,
    _phantom: PhantomData<C>,
}

impl<C, F> ConfigStoreClient<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    pub fn new(rpc_factory: Arc<F>, chain: Chain) -> Self {
        Self {
            contract: ACROSS_CONFIG_STORE.into(),
            rpc_factory,
            chain,
            _phantom: PhantomData,
        }
    }

    pub fn config_call3(&self, l1token: &Address) -> IMulticall3::Call3 {
        IMulticall3::Call3 {
            target: self.contract.parse().unwrap(),
            allowFailure: true,
            callData: AcrossConfigStore::l1TokenConfigCall { l1Token: *l1token }.abi_encode().into(),
        }
    }

    pub fn decoded_config_call3(&self, result: &IMulticall3::Result) -> Result<TokenConfig, SwapperError> {
        if result.success {
            let decoded = AcrossConfigStore::l1TokenConfigCall::abi_decode_returns(&result.returnData).map_err(SwapperError::from)?;
            let result: TokenConfig = serde_json::from_str(&decoded).map_err(SwapperError::from)?;
            Ok(result)
        } else {
            Err(SwapperError::ABIError("config call failed".into()))
        }
    }

    pub async fn fetch_config(&self, l1token: &Address) -> Result<TokenConfig, SwapperError> {
        let client = self.rpc_factory.client_for(self.chain).map_err(SwapperError::from)?;
        let data = AcrossConfigStore::l1TokenConfigCall { l1Token: *l1token }.abi_encode();
        let call = EthereumRpc::Call(TransactionObject::new_call(&self.contract, data), BlockParameter::Latest);
        let response: JsonRpcResult<String> = client.call_with_cache(&call, Some(CONFIG_CACHE_TTL)).await?;
        let result = response.take()?;
        let hex_data = HexDecode(result).map_err(|e| SwapperError::NetworkError(e.to_string()))?;
        let decoded = AcrossConfigStore::l1TokenConfigCall::abi_decode_returns(&hex_data).map_err(SwapperError::from)?;

        let result: TokenConfig = serde_json::from_str(&decoded).map_err(SwapperError::from)?;
        Ok(result)
    }
}
