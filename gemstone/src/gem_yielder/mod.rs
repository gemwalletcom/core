mod remote_types;
pub use remote_types::*;

use std::{collections::HashMap, sync::Arc};

use crate::{
    GemstoneError,
    alien::{AlienProvider, AlienProviderWrapper},
    models::{GemEarnData, GemEarnType, GemTransactionInputType, GemTransactionLoadInput, GemTransactionLoadMetadata},
};
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::rpc::RpcClient;
use primitives::{Chain, EVMChain};
use yielder::{GAS_LIMIT, YO_GATEWAY, YieldProvider, YieldProviderClient, Yielder, YoGatewayClient, YoProvider, YoYieldProvider};

pub(crate) fn build_yielder(rpc_provider: Arc<dyn AlienProvider>) -> Yielder {
    let wrapper = Arc::new(AlienProviderWrapper { provider: rpc_provider.clone() });

    let build_gateway = |chain: Chain, evm_chain: EVMChain| -> Option<(Chain, Arc<dyn YoProvider>)> {
        let endpoint = rpc_provider.get_endpoint(chain).ok()?;
        let rpc_client = RpcClient::new(endpoint, wrapper.clone());
        let ethereum_client = EthereumClient::new(JsonRpcClient::new(rpc_client), evm_chain);
        Some((chain, Arc::new(YoGatewayClient::new(ethereum_client, YO_GATEWAY)) as Arc<dyn YoProvider>))
    };

    let gateways: HashMap<Chain, Arc<dyn YoProvider>> = [build_gateway(Chain::Base, EVMChain::Base), build_gateway(Chain::Ethereum, EVMChain::Ethereum)]
        .into_iter()
        .flatten()
        .collect();

    let yo_provider: Arc<dyn YieldProviderClient> = Arc::new(YoYieldProvider::new(gateways));
    Yielder::new(vec![yo_provider])
}

pub(crate) async fn prepare_yield_input(yielder: &Yielder, input: GemTransactionLoadInput) -> Result<GemTransactionLoadInput, GemstoneError> {
    match (&input.input_type, &input.metadata) {
        (GemTransactionInputType::Earn { asset, earn_type }, GemTransactionLoadMetadata::Evm { nonce, chain_id, earn_data: None }) => {
            let provider = earn_type.provider_id().parse::<YieldProvider>().map_err(|e| GemstoneError::from(e.to_string()))?;
            let transaction = match earn_type {
                GemEarnType::Deposit(_) => yielder.deposit(provider, &asset.id, &input.sender_address, &input.value).await?,
                GemEarnType::Withdraw(_) => yielder.withdraw(provider, &asset.id, &input.sender_address, &input.value).await?,
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
