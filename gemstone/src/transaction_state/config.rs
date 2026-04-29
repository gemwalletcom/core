use primitives::{Chain, JobConfiguration};

const INITIAL_CAP_MS: u32 = 5_000;
const MAX_INTERVAL_MS: u32 = 15_000;
const STEP_FACTOR: f32 = 1.1;

#[uniffi::remote(Record)]
pub struct JobConfiguration {
    pub initial_interval_ms: u32,
    pub max_interval_ms: u32,
    pub step_factor: f32,
}

#[uniffi::export]
pub fn transaction_state_config(chain: Chain) -> JobConfiguration {
    let block_time_ms = chain.block_time();
    JobConfiguration {
        initial_interval_ms: block_time_ms.clamp(1, INITIAL_CAP_MS),
        max_interval_ms: MAX_INTERVAL_MS,
        step_factor: STEP_FACTOR,
    }
}
