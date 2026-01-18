mod remote_types;
pub use remote_types::*;

use std::{collections::HashMap, sync::Arc};

use crate::{
    GemstoneError,
    alien::{AlienProvider, AlienProviderWrapper},
    models::{GemTransactionInputType, GemTransactionLoadInput, GemYieldData},
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
        let yielder = build_yielder(rpc_provider)?;
        Ok(Self { yielder })
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

    pub async fn build_transaction(
        &self,
        action: GemYieldAction,
        provider: String,
        asset: AssetId,
        wallet_address: String,
        value: String,
        nonce: u64,
        chain_id: u64,
    ) -> Result<GemYieldTransactionData, GemstoneError> {
        let provider = provider.parse::<YieldProvider>()?;

        let transaction = match action {
            GemYieldAction::Deposit => {
                self.yielder.deposit(provider, &asset, &wallet_address, &value).await?
            }
            GemYieldAction::Withdraw => {
                self.yielder.withdraw(provider, &asset, &wallet_address, &value).await?
            }
        };

        Ok(GemYieldTransactionData {
            transaction,
            nonce,
            chain_id,
            gas_limit: "300000".to_string(),
        })
    }

}

pub(crate) fn build_yielder(rpc_provider: Arc<dyn AlienProvider>) -> Result<Yielder, GemstoneError> {
    let wrapper = Arc::new(AlienProviderWrapper { provider: rpc_provider.clone() });

    let build_gateway = |chain: Chain, evm_chain: EVMChain| -> Result<Arc<dyn YoProvider>, GemstoneError> {
        let endpoint = rpc_provider.get_endpoint(chain)?;
        let rpc_client = RpcClient::new(endpoint, wrapper.clone());
        let ethereum_client = EthereumClient::new(JsonRpcClient::new(rpc_client), evm_chain);
        Ok(Arc::new(YoGatewayClient::new(ethereum_client, YO_GATEWAY)))
    };

    let gateways: HashMap<Chain, Arc<dyn YoProvider>> = HashMap::from([
        (Chain::Base, build_gateway(Chain::Base, EVMChain::Base)?),
        (Chain::Ethereum, build_gateway(Chain::Ethereum, EVMChain::Ethereum)?),
    ]);

    let yo_provider: Arc<dyn YieldProviderClient> = Arc::new(YoYieldProvider::new(gateways));
    let mut yielder = Yielder::new();
    yielder.add_provider_arc(yo_provider);
    Ok(yielder)
}

pub(crate) async fn prepare_yield_input(
    yielder: &Yielder,
    input: GemTransactionLoadInput,
) -> Result<GemTransactionLoadInput, GemstoneError> {
    match &input.input_type {
        GemTransactionInputType::Yield { asset, action, data } => {
            if data.contract_address.is_empty() || data.call_data.is_empty() {
                let transaction = match action {
                    GemYieldAction::Deposit => {
                        yielder.deposit(YieldProvider::Yo, &asset.id, &input.sender_address, &input.value).await?
                    }
                    GemYieldAction::Withdraw => {
                        yielder.withdraw(YieldProvider::Yo, &asset.id, &input.sender_address, &input.value).await?
                    }
                };

                Ok(GemTransactionLoadInput {
                    input_type: GemTransactionInputType::Yield {
                        asset: asset.clone(),
                        action: action.clone(),
                        data: GemYieldData {
                            provider_name: data.provider_name.clone(),
                            contract_address: transaction.to,
                            call_data: transaction.data,
                            approval: transaction.approval,
                            gas_limit: Some("350000".to_string()),
                        },
                    },
                    sender_address: input.sender_address,
                    destination_address: input.destination_address,
                    value: input.value,
                    gas_price: input.gas_price,
                    memo: input.memo,
                    is_max_value: input.is_max_value,
                    metadata: input.metadata,
                })
            } else {
                Ok(input)
            }
        }
        _ => Ok(input),
    }
}
