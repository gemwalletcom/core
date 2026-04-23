use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use gem_tracing::info_with_fields;
use job_runner::pool::{AdaptiveConfig, Job, JobConfig, JobStatus};
use primitives::{TransactionId, TransactionState, TransactionStateInput, TransactionUpdate, chain_transaction_timeout};

use crate::error::StateError;
use crate::sources::{ChainStateSource, TransactionUpdateSink};

const INITIAL_INTERVAL_MAX: Duration = Duration::from_secs(5);
const MAX_INTERVAL: Duration = Duration::from_secs(15);
const STEP_FACTOR: f64 = 1.1;

pub struct TransactionJob {
    id: String,
    input: TransactionStateInput,
    chain_source: Arc<dyn ChainStateSource>,
    sink: Arc<dyn TransactionUpdateSink>,
}

impl TransactionJob {
    pub fn new(input: TransactionStateInput, chain_source: Arc<dyn ChainStateSource>, sink: Arc<dyn TransactionUpdateSink>) -> Self {
        let id = TransactionId::new(input.chain, input.hash.clone()).to_string();
        Self {
            id,
            input,
            chain_source,
            sink,
        }
    }

    fn is_timed_out(&self, now_secs: i64) -> bool {
        let elapsed_secs = (now_secs - self.input.created_at_secs).max(0) as u64;
        let timeout_secs = chain_transaction_timeout(self.input.chain) as u64 / 1000;
        elapsed_secs > timeout_secs
    }

    async fn emit(&self, update: TransactionUpdate) {
        self.sink.on_update(self.id.clone(), update).await;
    }
}

#[async_trait]
impl Job for TransactionJob {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn config(&self) -> JobConfig {
        let initial = Duration::from_millis(self.input.chain.block_time() as u64).min(INITIAL_INTERVAL_MAX);
        JobConfig::Adaptive {
            config: AdaptiveConfig::new(initial, MAX_INTERVAL, STEP_FACTOR),
            deadline: None,
        }
    }

    async fn run(&self) -> JobStatus {
        let now_secs = Utc::now().timestamp();
        match self.chain_source.get_transaction_status(&self.input).await {
            Ok(update) if update.state.is_completed() => {
                self.emit(update).await;
                JobStatus::Complete
            }
            Ok(_) if self.is_timed_out(now_secs) => {
                self.emit(TransactionUpdate::new(TransactionState::Failed, vec![])).await;
                JobStatus::Complete
            }
            Ok(_) => JobStatus::Retry,
            Err(StateError::NetworkError(_)) => JobStatus::Retry,
            Err(StateError::PlatformError(message)) => {
                info_with_fields!("transaction_state platform error", id = self.id.as_str(), msg = message.as_str());
                JobStatus::Complete
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use primitives::Chain;

    struct NoopChain;

    #[async_trait]
    impl ChainStateSource for NoopChain {
        async fn get_transaction_status(&self, _input: &TransactionStateInput) -> Result<TransactionUpdate, StateError> {
            Ok(TransactionUpdate::new(TransactionState::Pending, vec![]))
        }
    }

    struct NoopSink;

    #[async_trait]
    impl TransactionUpdateSink for NoopSink {
        async fn on_update(&self, _id: String, _update: TransactionUpdate) {}
    }

    fn job(created_at_secs: i64) -> TransactionJob {
        let input = TransactionStateInput {
            chain: Chain::Ethereum,
            hash: "0xabc".into(),
            from: "0xfrom".into(),
            created_at_secs,
            block_number: 0,
            swap_metadata: None,
        };
        TransactionJob::new(input, Arc::new(NoopChain), Arc::new(NoopSink))
    }

    #[test]
    fn test_is_timed_out() {
        let created_at = 1_700_000_000;
        let job = job(created_at);

        assert!(!job.is_timed_out(created_at + 1));
        assert!(!job.is_timed_out(created_at - 100));
        assert!(job.is_timed_out(created_at + 60 * 60 * 24 * 365));
    }
}
