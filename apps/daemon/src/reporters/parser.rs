use std::sync::Arc;

use primitives::Chain;

use crate::metrics::parser::ParserMetrics;

pub struct ParserReporter {
    chain: Chain,
    metrics: Arc<ParserMetrics>,
}

impl ParserReporter {
    pub fn new(chain: Chain, metrics: Arc<ParserMetrics>) -> Self {
        Self { chain, metrics }
    }

    pub fn error(&self, error: &str) {
        self.metrics.record_error(self.chain.as_ref(), error);
    }

    pub fn update_state(&self, current_block: i64, latest_block: i64, is_enabled: bool) {
        self.metrics.update_state(self.chain.as_ref(), current_block, latest_block, is_enabled);
    }
}
