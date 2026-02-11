mod remote_types;
pub use remote_types::*;

use std::{collections::HashMap, sync::Arc};

use crate::{
    GemstoneError,
    alien::{AlienProvider, AlienProviderWrapper},
    models::{GemDelegationBase, GemEarnData, GemEarnType, GemTransactionInputType, GemTransactionLoadInput, GemTransactionLoadMetadata},
};
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::rpc::RpcClient;
use primitives::{AssetId, Chain, EVMChain};
use yielder::{GAS_LIMIT, YO_GATEWAY, YieldDetailsRequest, YieldProvider, YieldProviderClient, Yielder, YoGatewayClient, YoProvider, YoYieldProvider};

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

    pub async fn deposit(&self, provider: GemYieldProvider, asset: AssetId, wallet_address: String, value: String) -> Result<GemEarnTransaction, GemstoneError> {
        self.yielder.deposit(provider, &asset, &wallet_address, &value).await.map_err(Into::into)
    }

    pub async fn withdraw(&self, provider: GemYieldProvider, asset: AssetId, wallet_address: String, value: String) -> Result<GemEarnTransaction, GemstoneError> {
        self.yielder.withdraw(provider, &asset, &wallet_address, &value).await.map_err(Into::into)
    }

    pub async fn positions(&self, provider: GemYieldProvider, asset: AssetId, wallet_address: String) -> Result<GemDelegationBase, GemstoneError> {
        let request = YieldDetailsRequest { asset_id: asset, wallet_address };
        self.yielder.positions(provider, &request).await.map_err(Into::into)
    }

    pub async fn build_transaction(
        &self,
        action: GemEarnType,
        provider: GemYieldProvider,
        asset: AssetId,
        wallet_address: String,
        value: String,
        nonce: u64,
        chain_id: u64,
    ) -> Result<GemEarnTransactionData, GemstoneError> {
        let transaction = match action {
            GemEarnType::Deposit(_) => self.yielder.deposit(provider, &asset, &wallet_address, &value).await?,
            GemEarnType::Withdraw(_) => self.yielder.withdraw(provider, &asset, &wallet_address, &value).await?,
        };

        Ok(GemEarnTransactionData {
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
    match (&input.input_type, &input.metadata) {
        (
            GemTransactionInputType::Earn { asset, earn_type },
            GemTransactionLoadMetadata::Evm { nonce, chain_id, earn_data: None },
        ) => {
            let transaction = match earn_type {
                GemEarnType::Deposit(_) => yielder.deposit(YieldProvider::Yo, &asset.id, &input.sender_address, &input.value).await?,
                GemEarnType::Withdraw(_) => yielder.withdraw(YieldProvider::Yo, &asset.id, &input.sender_address, &input.value).await?,
            };

            Ok(GemTransactionLoadInput {
                input_type: input.input_type.clone(),
                sender_address: input.sender_address,
                destination_address: input.destination_address,
                value: input.value,
                gas_price: input.gas_price,
                memo: input.memo,
                is_max_value: input.is_max_value,
                metadata: GemTransactionLoadMetadata::Evm {
                    nonce: *nonce,
                    chain_id: *chain_id,
                    earn_data: Some(GemEarnData {
                        provider: Some(earn_type.provider_id().to_string()),
                        contract_address: Some(transaction.to),
                        call_data: Some(transaction.data),
                        approval: transaction.approval,
                        gas_limit: Some(GAS_LIMIT.to_string()),
                    }),
                },
            })
        }
        _ => Ok(input),
    }
}
