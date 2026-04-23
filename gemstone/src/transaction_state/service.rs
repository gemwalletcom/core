use primitives::TransactionStateInput;
use std::sync::Arc;
use transaction_state::TransactionStateService;

use crate::alien::AlienProvider;
use crate::gateway::{ChainClientFactory, EmptyPreferences};

use super::listener::{ListenerSink, TransactionUpdateListener};
use super::sources::GatewayChainStateSource;

#[derive(uniffi::Object)]
pub struct GemTransactionStateService {
    inner: TransactionStateService,
}

#[uniffi::export(async_runtime = "tokio")]
impl GemTransactionStateService {
    #[uniffi::constructor]
    pub fn new(rpc_provider: Arc<dyn AlienProvider>, listener: Arc<dyn TransactionUpdateListener>) -> Arc<Self> {
        let factory = Arc::new(ChainClientFactory::new(rpc_provider, Arc::new(EmptyPreferences), Arc::new(EmptyPreferences)));
        let chain_source = Arc::new(GatewayChainStateSource { factory });
        let sink = Arc::new(ListenerSink { inner: listener });
        Arc::new(Self {
            inner: TransactionStateService::new(chain_source, sink),
        })
    }

    pub async fn monitor(&self, inputs: Vec<TransactionStateInput>) {
        self.inner.monitor(inputs).await
    }

    pub async fn stop_monitoring(&self) {
        self.inner.stop_monitoring().await
    }
}
