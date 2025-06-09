use crate::{
    network::{AlienProvider, JsonRpcClient, JsonRpcResult},
    swapper::SwapperError,
};
use alloy_primitives::{hex::decode as HexDecode, Address};
use alloy_sol_types::SolCall;
use gem_evm::{
    across::{contracts::AcrossConfigStore, deployment::ACROSS_CONFIG_STORE, fees},
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    multicall3::IMulticall3,
};
use primitives::Chain;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

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

pub struct ConfigStoreClient {
    pub contract: String,
    pub client: JsonRpcClient,
}

impl ConfigStoreClient {
    pub fn new(provider: Arc<dyn AlienProvider>, chain: Chain) -> ConfigStoreClient {
        ConfigStoreClient {
            contract: ACROSS_CONFIG_STORE.into(),
            client: JsonRpcClient::new_with_chain(provider.clone(), chain),
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
        let data = AcrossConfigStore::l1TokenConfigCall { l1Token: *l1token }.abi_encode();
        let call = EthereumRpc::Call(TransactionObject::new_call(&self.contract, data), BlockParameter::Latest);
        let response: JsonRpcResult<String> = self.client.call_with_cache(&call, Some(CONFIG_CACHE_TTL)).await?;
        let result = response.take()?;
        let hex_data = HexDecode(result).map_err(|e| SwapperError::NetworkError(e.to_string()))?;
        let decoded = AcrossConfigStore::l1TokenConfigCall::abi_decode_returns(&hex_data).map_err(SwapperError::from)?;

        let result: TokenConfig = serde_json::from_str(&decoded).map_err(SwapperError::from)?;
        Ok(result)
    }
}
