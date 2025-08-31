use crate::models::referral::HypercoreReferral;
use crate::models::user::{HypercoreAgentSession, HypercoreUserFee};
use crate::provider::preload_cache::HyperCoreCache;
use num_bigint::BigInt;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;

pub async fn get_approvals_and_credentials(
    cache: &HyperCoreCache,
    sender_address: &str,
    secure_preferences: Arc<dyn primitives::Preferences>,
    get_agents: impl Future<Output = Result<Vec<HypercoreAgentSession>, Box<dyn Error + Send + Sync>>>,
    get_referral: impl Future<Output = Result<HypercoreReferral, Box<dyn Error + Send + Sync>>>,
    get_builder_fee: impl Future<Output = Result<u32, Box<dyn Error + Send + Sync>>>,
    get_user_fees: impl Future<Output = Result<HypercoreUserFee, Box<dyn Error + Send + Sync>>>,
) -> Result<(bool, bool, bool, i64, String, String), Box<dyn Error + Send + Sync>> {
    let ((agent_required, agent_address, agent_private_key), referral_required, builder_required, fee_rate) = futures::try_join!(
        cache.manage_agent(sender_address, secure_preferences.clone(), get_agents),
        cache.needs_referral_approval(sender_address, get_referral),
        cache.needs_builder_fee_approval(sender_address, get_builder_fee),
        cache.get_user_fee_rate(sender_address, get_user_fees),
    )?;

    Ok((agent_required, referral_required, builder_required, fee_rate, agent_address, agent_private_key))
}

pub fn calculate_fee_amount(fiat_value: f64, fee_rate: i64) -> BigInt {
    let fee_rate_f64 = fee_rate as f64;
    let result = fiat_value * (fee_rate_f64 / 1_000_000.0) * 2.0 * 10.0 * 1_000_000.0;
    BigInt::from(result as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fee_amount_basic() {
        // 50000 thousands bps
        // 100.0 * (50000 / 1_000_000) * 2.0 * 10.0 * 1_000_000 = 100.0 * 0.05 * 20 * 1_000_000 = 100_000_000
        let result = calculate_fee_amount(100.0, 50000);
        assert_eq!(result, BigInt::from(100_000_000));
    }

    #[test]
    fn test_calculate_fee_amount_zero_fee_rate() {
        let result = calculate_fee_amount(1000.0, 0);
        assert_eq!(result, BigInt::from(0));
    }

    #[test]
    fn test_calculate_fee_amount_zero_fiat_value() {
        let result = calculate_fee_amount(0.0, 25000);
        assert_eq!(result, BigInt::from(0));
    }

    #[test]
    fn test_calculate_fee_amount_small_values() {
        // 1000 thousands bps
        // 1.0 * (1000 / 1_000_000) * 2.0 * 10.0 * 1_000_000 = 1.0 * 0.001 * 20 * 1_000_000 = 20_000
        let result = calculate_fee_amount(1.0, 1000);
        assert_eq!(result, BigInt::from(20_000));
    }

    #[test]
    fn test_calculate_fee_amount_large_values() {
        // 100000 thousands bps
        // 10000.0 * (100000 / 1_000_000) * 2.0 * 10.0 * 1_000_000 = 10000.0 * 0.1 * 20 * 1_000_000 = 20_000_000_000
        let result = calculate_fee_amount(10000.0, 100000);
        assert_eq!(result, BigInt::from(20_000_000_000_i64));
    }

    #[test]
    fn test_calculate_fee_amount_realistic_scenario() {
        // 30000 thousands bps
        // 500.0 * (30000 / 1_000_000) * 2.0 * 10.0 * 1_000_000 = 500.0 * 0.03 * 20 * 1_000_000 = 300_000_000
        let result = calculate_fee_amount(500.0, 30000);
        assert_eq!(result, BigInt::from(300_000_000));
    }

    #[test]
    fn test_calculate_fee_amount_debug_case() {
        // fee_rate = 43 (in thousands of bps)
        // 150.0 * (43 / 1_000_000) * 2.0 * 10.0 * 1_000_000 = 150.0 * 0.000043 * 20 * 1_000_000 = 129_000
        let result = calculate_fee_amount(150.0, 43);
        assert_eq!(result, BigInt::from(129_000));
    }

    #[test]
    fn test_calculate_fee_amount_expected_case() {
        // Expected: 1000 fiat value should result in 0.86 USDC = 860,000 USDC units
        // 1000.0 * (43 / 1_000_000) * 2.0 * 10.0 * 1_000_000 = 1000.0 * 0.000043 * 20 * 1_000_000 = 860_000
        let result = calculate_fee_amount(1000.0, 43);
        assert_eq!(result, BigInt::from(860_000));
    }
}
