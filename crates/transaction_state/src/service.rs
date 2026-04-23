use std::sync::Arc;

use job_runner::pool::JobRunner;
use primitives::TransactionStateInput;

use crate::job::TransactionJob;
use crate::sources::{ChainStateSource, TransactionUpdateSink};

pub struct TransactionStateService {
    runner: JobRunner,
    chain_source: Arc<dyn ChainStateSource>,
    sink: Arc<dyn TransactionUpdateSink>,
}

impl TransactionStateService {
    pub fn new(chain_source: Arc<dyn ChainStateSource>, sink: Arc<dyn TransactionUpdateSink>) -> Self {
        Self {
            runner: JobRunner::new(),
            chain_source,
            sink,
        }
    }

    pub async fn monitor(&self, inputs: Vec<TransactionStateInput>) {
        let jobs = inputs.into_iter().map(|input| {
            let job = Arc::new(TransactionJob::new(input, self.chain_source.clone(), self.sink.clone()));
            self.runner.spawn(job)
        });
        futures::future::join_all(jobs).await;
    }

    pub async fn stop_monitoring(&self) {
        self.runner.stop_all().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::StateError;
    use async_trait::async_trait;
    use chrono::Utc;
    use job_runner::pool::{Job, JobStatus};
    use primitives::{Chain, TransactionChange, TransactionState, TransactionUpdate};
    use std::sync::Mutex;

    struct ChainMock {
        responses: Mutex<Vec<Result<TransactionUpdate, StateError>>>,
    }

    #[async_trait]
    impl ChainStateSource for ChainMock {
        async fn get_transaction_status(&self, _input: &TransactionStateInput) -> Result<TransactionUpdate, StateError> {
            let mut queue = self.responses.lock().unwrap();
            queue.remove(0)
        }
    }

    struct SinkMock {
        emitted: Mutex<Vec<(String, TransactionUpdate)>>,
    }

    #[async_trait]
    impl TransactionUpdateSink for SinkMock {
        async fn on_update(&self, id: String, update: TransactionUpdate) {
            self.emitted.lock().unwrap().push((id, update));
        }
    }

    fn input(secs_ago: i64) -> TransactionStateInput {
        TransactionStateInput {
            chain: Chain::Ethereum,
            hash: "0xabc".into(),
            from: "0xfrom".into(),
            created_at_secs: Utc::now().timestamp() - secs_ago,
            block_number: 0,
            swap_metadata: None,
        }
    }

    fn mocks(responses: Vec<Result<TransactionUpdate, StateError>>) -> (Arc<ChainMock>, Arc<SinkMock>) {
        let chain = Arc::new(ChainMock { responses: Mutex::new(responses) });
        let sink = Arc::new(SinkMock { emitted: Mutex::new(vec![]) });
        (chain, sink)
    }

    #[tokio::test]
    async fn test_terminal_emits_and_completes() {
        let (chain, sink) = mocks(vec![Ok(TransactionUpdate::new(
            TransactionState::Confirmed,
            vec![TransactionChange::BlockNumber("123".into())],
        ))]);
        let job = TransactionJob::new(input(10), chain, sink.clone());

        let status = job.run().await;

        assert_eq!(status, JobStatus::Complete);
        let emitted = sink.emitted.lock().unwrap();
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].0, "ethereum_0xabc");
        assert_eq!(emitted[0].1.state, TransactionState::Confirmed);
    }

    #[tokio::test]
    async fn test_pending_past_deadline_emits_failed() {
        let (chain, sink) = mocks(vec![Ok(TransactionUpdate::new(TransactionState::Pending, vec![]))]);
        let job = TransactionJob::new(input(10_000_000), chain, sink.clone());

        let status = job.run().await;

        assert_eq!(status, JobStatus::Complete);
        let emitted = sink.emitted.lock().unwrap();
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].1.state, TransactionState::Failed);
    }
}
