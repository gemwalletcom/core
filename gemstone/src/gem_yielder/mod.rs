mod remote_types;
pub use remote_types::*;

use std::{collections::HashMap, sync::Arc};

use crate::{
    GemstoneError,
    alien::{AlienProvider, AlienProviderWrapper},
};
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::rpc::RpcClient;
use primitives::{AssetId, Chain, EVMChain};
use yielder::{
    YO_GATEWAY, YieldDetailsRequest, YieldProvider, YieldProviderClient, Yielder, YoGatewayClient, YoProvider, YoYieldProvider,
};

#[derive(uniffi::Object)]
pub struct GemYielder {
    yielder: Yielder,
}

impl std::fmt::Debug for GemYielder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GemYielder").finish()
    }
}

#[uniffi::export]
impl GemYielder {
    #[uniffi::constructor]
    pub fn new(rpc_provider: Arc<dyn AlienProvider>) -> Result<Self, GemstoneError> {
        let mut inner = Yielder::new();
        let yo_provider = build_yo_provider(rpc_provider)?;
        inner.add_provider_arc(yo_provider);
        Ok(Self { yielder: inner })
    }

    pub async fn yields_for_asset(&self, asset_id: &AssetId) -> Result<Vec<GemYield>, GemstoneError> {
        self.yielder.yields_for_asset_with_apy(asset_id).await.map_err(Into::into)
    }

    pub fn is_yield_available(&self, asset_id: &AssetId) -> bool {
        self.yielder.is_yield_available(asset_id)
    }

    pub async fn deposit(&self, provider: String, asset: AssetId, wallet_address: String, value: String) -> Result<GemYieldTransaction, GemstoneError> {
        let provider = provider.parse::<YieldProvider>()?;
        self.yielder.deposit(provider, &asset, &wallet_address, &value).await.map_err(Into::into)
    }

    pub async fn withdraw(&self, provider: String, asset: AssetId, wallet_address: String, value: String) -> Result<GemYieldTransaction, GemstoneError> {
        let provider = provider.parse::<YieldProvider>()?;
        self.yielder.withdraw(provider, &asset, &wallet_address, &value).await.map_err(Into::into)
    }

    pub async fn positions(&self, provider: String, asset: AssetId, wallet_address: String) -> Result<GemYieldPosition, GemstoneError> {
        let provider = provider.parse::<YieldProvider>()?;
        let request = YieldDetailsRequest {
            asset_id: asset,
            wallet_address,
        };
        self.yielder.positions(provider, &request).await.map_err(Into::into)
    }
}

fn build_yo_provider(rpc_provider: Arc<dyn AlienProvider>) -> Result<Arc<dyn YieldProviderClient>, GemstoneError> {
    let wrapper = Arc::new(AlienProviderWrapper {
        provider: rpc_provider.clone(),
    });
    let mut gateways: HashMap<Chain, Arc<dyn YoProvider>> = HashMap::new();

    // Base gateway
    let base_endpoint = rpc_provider.get_endpoint(Chain::Base)?;
    let base_rpc_client = RpcClient::new(base_endpoint, wrapper.clone());
    let base_jsonrpc_client = JsonRpcClient::new(base_rpc_client);
    let base_ethereum_client = EthereumClient::new(base_jsonrpc_client, EVMChain::Base);
    let base_gateway: Arc<dyn YoProvider> = Arc::new(YoGatewayClient::new(base_ethereum_client, YO_GATEWAY));
    gateways.insert(Chain::Base, base_gateway);

    // Ethereum gateway
    let eth_endpoint = rpc_provider.get_endpoint(Chain::Ethereum)?;
    let eth_rpc_client = RpcClient::new(eth_endpoint, wrapper);
    let eth_jsonrpc_client = JsonRpcClient::new(eth_rpc_client);
    let eth_ethereum_client = EthereumClient::new(eth_jsonrpc_client, EVMChain::Ethereum);
    let eth_gateway: Arc<dyn YoProvider> = Arc::new(YoGatewayClient::new(eth_ethereum_client, YO_GATEWAY));
    gateways.insert(Chain::Ethereum, eth_gateway);

    let provider: Arc<dyn YieldProviderClient> = Arc::new(YoYieldProvider::new(gateways));
    Ok(provider)
}
