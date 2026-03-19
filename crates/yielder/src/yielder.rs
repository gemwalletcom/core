use std::collections::HashMap;
use std::sync::Arc;

use gem_jsonrpc::alien::RpcProvider;
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Chain, ContractCallData, DelegationBase, DelegationValidator, EarnType};

use crate::error::YielderError;
use crate::provider::EarnProvider;
use crate::yo::YoEarnProvider;

pub struct Yielder {
    providers: Vec<Arc<dyn EarnProvider>>,
}

impl Yielder {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::with_providers(vec![Arc::new(YoEarnProvider::new(rpc_provider))])
    }

    pub fn with_providers(providers: Vec<Arc<dyn EarnProvider>>) -> Self {
        Self { providers }
    }

    pub fn get_providers(&self, asset_id: &AssetId) -> Vec<DelegationValidator> {
        self.providers.iter().filter_map(|p| p.get_provider(asset_id)).collect()
    }

    pub async fn get_positions(&self, address: &str, asset_id: &AssetId) -> Vec<DelegationBase> {
        let futures: Vec<_> = self.providers.iter().map(|p| p.get_position(address, asset_id)).collect();
        futures::future::join_all(futures).await.into_iter().filter_map(|r| r.ok().flatten()).collect()
    }

    pub async fn get_balance(&self, chain: Chain, address: &str, token_ids: &[String]) -> Vec<AssetBalance> {
        let futures: Vec<_> = self.providers.iter().map(|p| p.get_balance(chain, address, token_ids)).collect();
        let balances = futures::future::join_all(futures).await.into_iter().filter_map(|r| r.ok()).flatten().collect();
        Self::map_earn_balances(balances)
    }

    pub async fn get_data(&self, asset_id: &AssetId, address: &str, value: &str, earn_type: &EarnType) -> Result<ContractCallData, YielderError> {
        self.providers
            .iter()
            .find(|p| p.get_provider(asset_id).is_some_and(|v| v.id == earn_type.provider_id()))
            .ok_or(YielderError::NotSupportedAsset)?
            .get_data(asset_id, address, value, earn_type)
            .await
    }

    fn map_earn_balances(balances: Vec<AssetBalance>) -> Vec<AssetBalance> {
        balances
            .into_iter()
            .fold(HashMap::<AssetId, BigUint>::new(), |mut acc, b| {
                *acc.entry(b.asset_id).or_default() += b.balance.earn;
                acc
            })
            .into_iter()
            .map(|(id, earn)| AssetBalance::new_earn(id, earn))
            .collect()
    }

}
