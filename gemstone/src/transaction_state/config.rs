use primitives::{Chain, JobConfiguration};

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GemJobConfiguration {
    pub initial_interval_ms: u32,
    pub max_interval_ms: u32,
    pub step_factor: f32,
}

#[uniffi::export]
pub fn transaction_state_config(chain: Chain) -> GemJobConfiguration {
    let config = JobConfiguration::transaction_state(chain);
    GemJobConfiguration {
        initial_interval_ms: config.initial_interval_ms,
        max_interval_ms: config.max_interval_ms,
        step_factor: config.step_factor,
    }
}
