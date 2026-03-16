use std::collections::HashMap;
use std::sync::Arc;

use gem_jsonrpc::{RpcClientError, RpcProvider};
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Chain, ContractCallData, DelegationBase, DelegationValidator, EarnType, YieldProvider};

use crate::client_factory::create_eth_client;
use crate::error::YielderError;
use crate::provider::EarnProvider;
use crate::yo::{YO_GATEWAY, YoClient, YoEarnProvider, YoGatewayClient, supported_assets};

pub struct Yielder {
    providers: Vec<Arc<dyn EarnProvider>>,
}

impl Yielder {
    pub fn new<E: RpcClientError + Clone + 'static>(rpc_provider: Arc<dyn RpcProvider<Error = E>>) -> Self {
        let gateways: HashMap<Chain, Arc<dyn YoClient>> = supported_assets()
            .iter()
            .filter_map(|asset| {
                let chain = asset.chain;
                let client = create_eth_client(rpc_provider.clone(), chain).ok()?;
                Some((chain, Arc::new(YoGatewayClient::new(client, YO_GATEWAY)) as Arc<dyn YoClient>))
            })
            .collect();

        let yo_provider: Arc<dyn EarnProvider> = Arc::new(YoEarnProvider::new(gateways));
        Self { providers: vec![yo_provider] }
    }

    pub fn get_providers(&self, asset_id: &AssetId) -> Vec<DelegationValidator> {
        self.providers.iter().flat_map(|p| p.get_providers(asset_id)).collect()
    }

    pub async fn get_positions(&self, chain: Chain, address: &str, asset_ids: &[AssetId]) -> Vec<DelegationBase> {
        let futures: Vec<_> = self.providers.iter().map(|p| p.get_positions(chain, address, asset_ids)).collect();
        futures::future::join_all(futures).await.into_iter().filter_map(|r| r.ok()).flatten().collect()
    }

    pub async fn get_balance(&self, chain: Chain, address: &str) -> Vec<AssetBalance> {
        let asset_ids: Vec<_> = self.providers.iter().flat_map(|p| p.get_asset_ids_for_chain(chain)).collect();
        let positions: HashMap<_, _> = self.get_positions(chain, address, &asset_ids).await.into_iter().map(|p| (p.asset_id, p.balance)).collect();

        asset_ids
            .into_iter()
            .map(|id| {
                let balance = positions.get(&id).cloned().unwrap_or(BigUint::ZERO);
                AssetBalance::new_earn(id, balance)
            })
            .collect()
    }

    pub async fn get_data(&self, asset_id: &AssetId, address: &str, value: &str, earn_type: &EarnType) -> Result<ContractCallData, YielderError> {
        let provider_id = earn_type.provider_id().parse::<YieldProvider>()?;
        let provider = self.provider_by_id(provider_id)?;
        match earn_type {
            EarnType::Deposit(_) => provider.deposit(asset_id, address, value).await,
            EarnType::Withdraw(delegation) => {
                let shares = delegation.base.shares.to_string();
                provider.withdraw(asset_id, address, value, &shares).await
            }
        }
    }

    fn provider_by_id(&self, provider: YieldProvider) -> Result<Arc<dyn EarnProvider>, YielderError> {
        self.providers
            .iter()
            .find(|p| p.id() == provider)
            .cloned()
            .ok_or_else(|| YielderError::provider_not_found(&provider))
    }
}
