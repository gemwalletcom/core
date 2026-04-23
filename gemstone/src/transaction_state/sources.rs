use async_trait::async_trait;
use primitives::{TransactionStateInput, TransactionStateRequest, TransactionUpdate};
use std::sync::Arc;
use transaction_state::{ChainStateSource, StateError};

use crate::gateway::ChainClientFactory;

pub(crate) struct GatewayChainStateSource {
    pub(crate) factory: Arc<ChainClientFactory>,
}

#[async_trait]
impl ChainStateSource for GatewayChainStateSource {
    async fn get_transaction_status(&self, input: &TransactionStateInput) -> Result<TransactionUpdate, StateError> {
        let provider = self.factory.create(input.chain).await.map_err(|e| StateError::PlatformError(e.to_string()))?;
        let request: TransactionStateRequest = input.into();
        provider.get_transaction_status(request).await.map_err(|e| StateError::NetworkError(e.to_string()))
    }
}
