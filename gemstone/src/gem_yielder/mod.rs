mod remote_types;
pub use remote_types::*;

use std::sync::Arc;

use crate::{
    GemstoneError,
    alien::{AlienProvider, AlienProviderWrapper},
};
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::rpc::RpcClient;
use primitives::{AssetId, Chain, EVMChain};
use yielder::{YO_GATEWAY_BASE_MAINNET, YieldDetailsRequest, YieldProvider, Yielder, YoGatewayApi, YoGatewayClient, YoYieldProvider};

#[derive(uniffi::Object)]
pub struct GemYielder {
    inner: Yielder,
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
        Ok(Self { inner })
    }

    pub async fn yields_for_asset(&self, asset_id: &AssetId) -> Result<Vec<GemYield>, GemstoneError> {
        self.inner.yields_for_asset_with_apy(asset_id).await.map_err(Into::into)
    }

    pub async fn deposit(&self, provider: String, asset: AssetId, wallet_address: String, amount: String) -> Result<GemYieldTransaction, GemstoneError> {
        self.inner.deposit(&provider, &asset, &wallet_address, &amount).await.map_err(Into::into)
    }

    pub async fn withdraw(&self, provider: String, asset: AssetId, wallet_address: String, amount: String) -> Result<GemYieldTransaction, GemstoneError> {
        self.inner.withdraw(&provider, &asset, &wallet_address, &amount).await.map_err(Into::into)
    }

    pub async fn details(&self, provider: String, asset: AssetId, wallet_address: String) -> Result<GemYieldDetails, GemstoneError> {
        let request = YieldDetailsRequest { asset, wallet_address };
        self.inner.details(&provider, &request).await.map_err(Into::into)
    }
}

fn build_yo_provider(rpc_provider: Arc<dyn AlienProvider>) -> Result<Arc<dyn YieldProvider>, GemstoneError> {
    let endpoint = rpc_provider.get_endpoint(Chain::Base)?;
    let wrapper = AlienProviderWrapper { provider: rpc_provider };
    let rpc_client = RpcClient::new(endpoint, Arc::new(wrapper));
    let jsonrpc_client = JsonRpcClient::new(rpc_client);
    let evm_chain = EVMChain::Base;
    let ethereum_client = EthereumClient::new(jsonrpc_client, evm_chain);
    let gateway_client = YoGatewayClient::new(ethereum_client, YO_GATEWAY_BASE_MAINNET);
    let gateway: Arc<dyn YoGatewayApi> = Arc::new(gateway_client);
    let provider: Arc<dyn YieldProvider> = Arc::new(YoYieldProvider::new(gateway));
    Ok(provider)
}
