use std::sync::Arc;

use gem_jsonrpc::{RpcProvider, Target};
use primitives::Chain;

use super::model::{YoApiResponse, YoPerformanceData};
use crate::yo::YieldError;

const YO_API_BASE_URL: &str = "https://api.yo.xyz";

pub struct YoApiClient<E: std::error::Error + Send + Sync + 'static> {
    rpc_provider: Arc<dyn RpcProvider<Error = E>>,
}

impl<E: std::error::Error + Send + Sync + 'static> YoApiClient<E> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider<Error = E>>) -> Self {
        Self { rpc_provider }
    }

    pub async fn fetch_rewards(&self, chain: Chain, vault_address: &str, user_address: &str) -> Result<YoPerformanceData, YieldError> {
        let network = match chain {
            Chain::Base => "base",
            Chain::Ethereum => "mainnet",
            _ => return Err(YieldError::new(format!("unsupported chain for Yo API: {:?}", chain))),
        };
        let url = format!("{}/api/v1/performance/user/{}/{}/{}", YO_API_BASE_URL, network, vault_address, user_address);
        let target = Target::get(&url);

        let response = self
            .rpc_provider
            .request(target)
            .await
            .map_err(|e| YieldError::new(format!("API request failed: {}", e)))?;

        let parsed: YoApiResponse<YoPerformanceData> =
            serde_json::from_slice(&response.data).map_err(|e| YieldError::new(format!("failed to parse Yo API response: {}", e)))?;

        if parsed.status_code != 200 {
            return Err(YieldError::new(format!("Yo API error: {}", parsed.message)));
        }

        Ok(parsed.data)
    }
}
