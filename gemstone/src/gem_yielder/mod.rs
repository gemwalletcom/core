mod remote_types;
pub use remote_types::*;

use std::sync::Arc;

use crate::{
    alien::{AlienProvider, AlienProviderWrapper},
    GemstoneError,
};
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::rpc::RpcClient;
use primitives::{AssetId, Chain, EVMChain};
use yielder::{YieldProvider, YoGatewayApi, YoGatewayClient, YoYieldProvider, Yielder, YO_GATEWAY_BASE_MAINNET};

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

    pub fn yields_for_asset(&self, asset_id: &AssetId) -> Vec<GemYield> {
        self.inner.yields_for_asset(asset_id)
    }

    pub async fn deposit(&self, provider: String, request: GemYieldDepositRequest) -> Result<GemYieldTransaction, GemstoneError> {
        self.inner.deposit(&provider, &request).await.map_err(Into::into)
    }

    pub async fn withdraw(&self, provider: String, request: GemYieldWithdrawRequest) -> Result<GemYieldTransaction, GemstoneError> {
        self.inner.withdraw(&provider, &request).await.map_err(Into::into)
    }

    pub async fn details(&self, provider: String, request: GemYieldDetailsRequest) -> Result<GemYieldDetails, GemstoneError> {
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
