use crate::models::referral::Referral;
use crate::models::user::{AgentSession, UserFee};
use crate::provider::preload_cache::HyperCoreCache;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;

pub async fn get_approvals_and_credentials(
    cache: &HyperCoreCache,
    sender_address: &str,
    secure_preferences: Arc<dyn primitives::Preferences>,
    get_agents: impl Future<Output = Result<Vec<AgentSession>, Box<dyn Error + Send + Sync>>>,
    get_referral: impl Future<Output = Result<Referral, Box<dyn Error + Send + Sync>>>,
    get_builder_fee: impl Future<Output = Result<u32, Box<dyn Error + Send + Sync>>>,
    get_user_fees: impl Future<Output = Result<UserFee, Box<dyn Error + Send + Sync>>>,
) -> Result<(bool, bool, bool, i64, String, String), Box<dyn Error + Send + Sync>> {
    let ((agent_required, agent_address, agent_private_key), referral_required, builder_required, fee_rate) = futures::try_join!(
        cache.manage_agent(sender_address, secure_preferences.clone(), get_agents),
        cache.needs_referral_approval(sender_address, get_referral),
        cache.needs_builder_fee_approval(sender_address, get_builder_fee),
        cache.get_user_fee_rate(sender_address, get_user_fees),
    )?;

    Ok((agent_required, referral_required, builder_required, fee_rate, agent_address, agent_private_key))
}
