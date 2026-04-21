use primitives::{Chain, chain_transaction_timeout};

const INITIAL_CAP_MS: u64 = 5_000;
const MAX_INTERVAL_MS: u64 = 15_000;
const STEP_FACTOR: f32 = 1.1;

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct TransactionStateConfig {
    pub initial_interval_ms: u64,
    pub max_interval_ms: u64,
    pub step_factor: f32,
    pub timeout_seconds: u64,
}

#[uniffi::export]
pub fn transaction_state_config(chain: Chain) -> TransactionStateConfig {
    let block_time_ms = chain.block_time() as u64;
    let timeout_ms = chain_transaction_timeout(chain) as u64;
    TransactionStateConfig {
        initial_interval_ms: block_time_ms.min(INITIAL_CAP_MS),
        max_interval_ms: MAX_INTERVAL_MS,
        step_factor: STEP_FACTOR,
        timeout_seconds: timeout_ms / 1000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_state_config() {
        let eth = transaction_state_config(Chain::Ethereum);
        assert_eq!(eth.max_interval_ms, MAX_INTERVAL_MS);
        assert!(eth.initial_interval_ms <= INITIAL_CAP_MS);
        assert!(eth.step_factor > 1.0);

        let btc = transaction_state_config(Chain::Bitcoin);
        assert_eq!(btc.initial_interval_ms, INITIAL_CAP_MS);
    }
}
