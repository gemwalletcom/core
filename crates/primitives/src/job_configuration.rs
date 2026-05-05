use crate::Chain;

const INITIAL_CAP_MS: u32 = 5_000;
const MAX_INTERVAL_MS: u32 = 15_000;
const STEP_FACTOR: f32 = 1.1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JobConfiguration {
    pub initial_interval_ms: u32,
    pub max_interval_ms: u32,
    pub step_factor: f32,
}

impl JobConfiguration {
    pub fn transaction_state(chain: Chain) -> Self {
        Self {
            initial_interval_ms: chain.block_time().clamp(1, INITIAL_CAP_MS),
            max_interval_ms: MAX_INTERVAL_MS,
            step_factor: STEP_FACTOR,
        }
    }
}
