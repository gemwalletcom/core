use crate::{
    network::{jsonrpc::jsonrpc_call_with_cache, AlienProvider, JsonRpcResult},
    swapper::SwapperError,
};
use alloy_core::sol_types::SolCall;
use alloy_primitives::Address;
use gem_evm::{
    across::{contracts::AcrossConfigStore, fees},
    address::EthereumAddress,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
};
use primitives::Chain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

const CONFIG_CACHE_TTL: u64 = 60 * 60 * 24;

#[derive(Serialize, Deserialize, Debug)]
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
    pub provider: Arc<dyn AlienProvider>,
    pub chain: Chain,
}

impl ConfigStoreClient {
    pub async fn fetch_config(&self, l1token: &EthereumAddress) -> Result<TokenConfig, SwapperError> {
        let data = AcrossConfigStore::l1TokenConfigCall {
            l1Token: Address::from_slice(&l1token.bytes),
        }
        .abi_encode();
        let call = EthereumRpc::Call(TransactionObject::new_call(&self.contract, data), BlockParameter::Latest);
        let response: JsonRpcResult<String> = jsonrpc_call_with_cache(&call, self.provider.clone(), &self.chain, Some(CONFIG_CACHE_TTL)).await?;
        let result = response.take()?;
        let result: TokenConfig = serde_json::from_str(&result).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        Ok(result)
    }
}
