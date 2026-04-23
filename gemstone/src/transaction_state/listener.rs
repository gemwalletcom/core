use async_trait::async_trait;
use primitives::TransactionUpdate;
use std::sync::Arc;
use transaction_state::TransactionUpdateSink;

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait TransactionUpdateListener: Send + Sync {
    async fn on_update(&self, id: String, update: TransactionUpdate);
}

pub(crate) struct ListenerSink {
    pub(crate) inner: Arc<dyn TransactionUpdateListener>,
}

#[async_trait]
impl TransactionUpdateSink for ListenerSink {
    async fn on_update(&self, id: String, update: TransactionUpdate) {
        self.inner.on_update(id, update).await;
    }
}
