mod remote_types;
pub use remote_types::*;

use std::{collections::HashMap, sync::Arc};

use crate::{
    GemstoneError,
    alien::{AlienProvider, AlienProviderWrapper},
    models::{GemEarnData, GemTransactionInputType, GemTransactionLoadInput},
};
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::rpc::RpcClient;
use primitives::{AssetId, Chain, EVMChain};
use yielder::{GAS_LIMIT, YO_GATEWAY, YieldDetailsRequest, YieldProvider, YieldProviderClient, YieldTransaction, Yielder, YoGatewayClient, YoProvider, YoYieldProvider};

#[derive(uniffi::Object)]
pub struct GemYielder {
    yielder: Yielder,
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

    pub async fn deposit(&self, provider: GemEarnProvider, asset: AssetId, wallet_address: String, value: String) -> Result<GemYieldTransaction, GemstoneError> {
        self.yielder.deposit(provider, &asset, &wallet_address, &value).await.map_err(Into::into)
    }

    pub async fn withdraw(&self, provider: GemEarnProvider, asset: AssetId, wallet_address: String, value: String) -> Result<GemYieldTransaction, GemstoneError> {
        self.yielder.withdraw(provider, &asset, &wallet_address, &value).await.map_err(Into::into)
    }

    pub async fn positions(&self, provider: GemEarnProvider, asset: AssetId, wallet_address: String) -> Result<GemEarnPosition, GemstoneError> {
        let request = YieldDetailsRequest { asset_id: asset, wallet_address };
        self.yielder.positions(provider, &request).await.map_err(Into::into)
    }

    pub async fn build_transaction(
        &self,
        action: GemEarnAction,
        provider: GemEarnProvider,
        asset: AssetId,
        wallet_address: String,
        value: String,
        nonce: u64,
        chain_id: u64,
    ) -> Result<GemYieldTransactionData, GemstoneError> {
        let transaction = build_yield_transaction(&self.yielder, &action, provider, &asset, &wallet_address, &value).await?;

        Ok(GemYieldTransactionData {
            transaction,
            nonce,
            chain_id,
            gas_limit: GAS_LIMIT.to_string(),
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
    Ok(Yielder::new(vec![yo_provider]))
}

pub(crate) async fn prepare_yield_input(yielder: &Yielder, input: GemTransactionLoadInput) -> Result<GemTransactionLoadInput, GemstoneError> {
    match &input.input_type {
        GemTransactionInputType::Earn { asset, action, data } => {
            if data.contract_address.is_none() || data.call_data.is_none() {
                let transaction = build_yield_transaction(yielder, action, YieldProvider::Yo, &asset.id, &input.sender_address, &input.value).await?;

                Ok(GemTransactionLoadInput {
                    input_type: GemTransactionInputType::Earn {
                        asset: asset.clone(),
                        action: action.clone(),
                        data: GemEarnData {
                            provider: data.provider.clone(),
                            contract_address: Some(transaction.to),
                            call_data: Some(transaction.data),
                            approval: transaction.approval,
                            gas_limit: Some(GAS_LIMIT.to_string()),
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

async fn build_yield_transaction(
    yielder: &Yielder,
    action: &GemEarnAction,
    provider: YieldProvider,
    asset: &AssetId,
    wallet_address: &str,
    value: &str,
) -> Result<YieldTransaction, GemstoneError> {
    match action {
        GemEarnAction::Deposit => Ok(yielder.deposit(provider, asset, wallet_address, value).await?),
        GemEarnAction::Withdraw => Ok(yielder.withdraw(provider, asset, wallet_address, value).await?),
    }
}
