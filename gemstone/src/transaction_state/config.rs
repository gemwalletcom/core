use primitives::Chain;

const INITIAL_CAP_MS: u32 = 5_000;
const MAX_INTERVAL_MS: u32 = 15_000;
const STEP_FACTOR: f32 = 1.1;

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GemJobConfiguration {
    pub initial_interval_ms: u32,
    pub max_interval_ms: u32,
    pub step_factor: f32,
}

#[uniffi::export]
pub fn transaction_state_config(chain: Chain) -> GemJobConfiguration {
    let block_time_ms = chain.block_time();
    GemJobConfiguration {
        initial_interval_ms: block_time_ms.clamp(1, INITIAL_CAP_MS),
        max_interval_ms: MAX_INTERVAL_MS,
        step_factor: STEP_FACTOR,
    }
}
