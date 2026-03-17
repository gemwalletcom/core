use std::sync::Arc;

use gem_jsonrpc::{RpcClientError, RpcProvider};
use primitives::{AssetBalance, AssetId, Chain, ContractCallData, DelegationBase, DelegationValidator, EarnType};

use crate::error::YielderError;
use crate::provider::EarnProvider;
use crate::yo::YoEarnProvider;

pub struct Yielder {
    providers: Vec<Arc<dyn EarnProvider>>,
}

impl Yielder {
    pub fn new<E: RpcClientError + Clone + 'static>(rpc_provider: Arc<dyn RpcProvider<Error = E>>) -> Self {
        Self {
            providers: vec![Arc::new(YoEarnProvider::new(rpc_provider))],
        }
    }

    pub fn get_provider(&self, asset_id: &AssetId) -> Option<DelegationValidator> {
        self.providers.iter().find_map(|p| p.get_provider(asset_id))
    }

    pub async fn get_position(&self, address: &str, asset_id: &AssetId) -> Option<DelegationBase> {
        let futures: Vec<_> = self.providers.iter().map(|p| p.get_position(address, asset_id)).collect();
        futures::future::join_all(futures).await.into_iter().find_map(|r| r.ok().flatten())
    }

    pub async fn get_balance(&self, chain: Chain, address: &str) -> Vec<AssetBalance> {
        let futures: Vec<_> = self.providers.iter().map(|p| p.get_balance(chain, address)).collect();
        futures::future::join_all(futures).await.into_iter().filter_map(|r| r.ok()).flatten().collect()
    }

    pub async fn get_data(&self, asset_id: &AssetId, address: &str, value: &str, earn_type: &EarnType) -> Result<ContractCallData, YielderError> {
        let provider = self.providers.iter().find(|p| p.get_provider(asset_id).is_some()).ok_or_else(|| YielderError::unsupported_asset(asset_id))?;
        provider.get_data(asset_id, address, value, earn_type).await
    }
}
