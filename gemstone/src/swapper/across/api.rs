use crate::{
    network::{AlienProvider, AlienTarget, EvmRpcClientFactory},
    swapper::SwapperError,
};
use gem_client::Client;
use gem_evm::rpc::{client::EthereumClient, model::TransactionReciept};
use primitives::{Chain, EVMChain, swap::SwapStatus};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData, sync::Arc};

#[derive(Debug, Clone)]
pub struct AcrossApi<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    pub url: String,
    pub provider: Arc<dyn AlienProvider>,
    rpc_factory: Arc<F>,
    _phantom: PhantomData<C>,
}

impl<C, F> AcrossApi<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    pub fn new(provider: Arc<dyn AlienProvider>, rpc_factory: Arc<F>) -> Self {
        Self {
            url: "https://app.across.to".into(),
            provider,
            rpc_factory,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositStatus {
    pub status: String,
    pub deposit_id: String,
    pub deposit_tx_hash: String,
    pub fill_tx: Option<String>,
    pub destination_chain_id: u64,
    pub deposit_refund_tx_hash: Option<String>,
}

impl DepositStatus {
    pub fn swap_status(&self) -> SwapStatus {
        match self.status.as_str() {
            "filled" => SwapStatus::Completed,
            "refunded" => SwapStatus::Refunded,
            _ => SwapStatus::Pending,
        }
    }
}

impl<C, F> AcrossApi<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    pub async fn deposit_status(&self, chain: Chain, tx_hash: &str) -> Result<DepositStatus, SwapperError> {
        let evm_chain = EVMChain::from_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        let client = self.rpc_factory.client_for(chain).map_err(SwapperError::from)?;
        let eth_client = EthereumClient::new(client, evm_chain);
        let receipt: TransactionReciept = eth_client
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(SwapperError::from)?;
        if receipt.logs.len() < 2 || receipt.logs[1].topics.len() < 4 {
            return Err(SwapperError::NetworkError("invalid tx receipt".into()));
        }
        let deposit_id = receipt.logs[1].topics[3].clone();
        let url = format!("{}/deposit/status?originChainId={}&depositId={}", self.url, chain.network_id(), &deposit_id);
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await?;
        let status: DepositStatus = serde_json::from_slice(&response).map_err(SwapperError::from)?;

        Ok(status)
    }
}
