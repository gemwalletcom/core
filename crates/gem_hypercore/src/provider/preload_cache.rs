use crate::config::HypercoreConfig;
use crate::models::referral::Referral;
use crate::models::user::{AgentSession, UserFee};
use crate::rpc::client::agent_owner_cache_key;
use primitives::{Preferences, PreferencesExt};
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct UserFeeRates {
    pub(crate) perpetual_cross: f64,
    pub(crate) spot_cross: f64,
}

pub(crate) struct HyperCoreCache {
    preferences: Arc<dyn Preferences>,
    config: HypercoreConfig,
}

impl HyperCoreCache {
    const REFERRAL_APPROVED_KEY: &'static str = "referral_approved";
    const BUILDER_FEE_APPROVED_KEY: &'static str = "builder_fee_approved";
    const AGENT_VALID_UNTIL_KEY: &'static str = "agent_valid_until";
    const USER_PERPETUAL_FEE_RATE_KEY: &'static str = "user_perpetual_fee_rate";
    const USER_SPOT_FEE_RATE_KEY: &'static str = "user_spot_fee_rate";
    const USER_FEES_TTL: u64 = 86_400 * 7;
    const USER_FEE_RATE_SCALE: f64 = 1_000_000_000.0;

    pub(crate) fn new(preferences: Arc<dyn Preferences>, config: HypercoreConfig) -> Self {
        Self { preferences, config }
    }

    fn cache_key(&self, address: &str, key: &str) -> String {
        format!("{}_{}", address, key)
    }

    fn current_time() -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64)
    }

    pub(crate) async fn needs_referral_approval<F>(&self, address: &str, checker: F) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        F: Future<Output = Result<Referral, Box<dyn Error + Send + Sync>>>,
    {
        let cache_key = self.cache_key(address, Self::REFERRAL_APPROVED_KEY);

        if let Some(true) = self.preferences.get_bool(&cache_key)? {
            return Ok(false);
        }

        let referral = checker.await?;
        let needs_approval = referral.referred_by.is_none() && referral.cum_vlm < 10000.0;

        if !needs_approval {
            self.preferences.set_bool(&cache_key, true)?;
        }

        Ok(needs_approval)
    }

    pub(crate) async fn needs_builder_fee_approval<F>(&self, address: &str, checker: F) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        F: Future<Output = Result<u32, Box<dyn Error + Send + Sync>>>,
    {
        let cache_key = self.cache_key(address, Self::BUILDER_FEE_APPROVED_KEY);

        if let Some(true) = self.preferences.get_bool(&cache_key)? {
            return Ok(false);
        }

        let fee = checker.await?;
        let needs_approval = self.config.max_builder_fee_bps > fee;

        if !needs_approval {
            self.preferences.set_bool(&cache_key, true)?;
        }

        Ok(needs_approval)
    }

    pub(crate) async fn get_user_fee_rates<F>(&self, address: &str, fetcher: F) -> Result<UserFeeRates, Box<dyn Error + Send + Sync>>
    where
        F: Future<Output = Result<UserFee, Box<dyn Error + Send + Sync>>>,
    {
        let perpetual_cache_key = self.cache_key(address, Self::USER_PERPETUAL_FEE_RATE_KEY);
        let spot_cache_key = self.cache_key(address, Self::USER_SPOT_FEE_RATE_KEY);

        if let (Some(perpetual_cross), Some(spot_cross)) = (
            self.preferences.get_i64_with_ttl(&perpetual_cache_key, Self::USER_FEES_TTL)?,
            self.preferences.get_i64_with_ttl(&spot_cache_key, Self::USER_FEES_TTL)?,
        ) {
            return Ok(UserFeeRates {
                perpetual_cross: perpetual_cross as f64 / Self::USER_FEE_RATE_SCALE,
                spot_cross: spot_cross as f64 / Self::USER_FEE_RATE_SCALE,
            });
        }

        let user_fees = fetcher.await?;
        let discount = 1.0 - user_fees.active_referral_discount;
        let perpetual_cross = (user_fees.user_cross_rate * discount * Self::USER_FEE_RATE_SCALE).round() as i64;
        let spot_cross = (user_fees.user_spot_cross_rate * discount * Self::USER_FEE_RATE_SCALE).round() as i64;

        self.preferences.set_i64_with_ttl(&perpetual_cache_key, perpetual_cross, Self::USER_FEES_TTL)?;
        self.preferences.set_i64_with_ttl(&spot_cache_key, spot_cross, Self::USER_FEES_TTL)?;
        Ok(UserFeeRates {
            perpetual_cross: perpetual_cross as f64 / Self::USER_FEE_RATE_SCALE,
            spot_cross: spot_cross as f64 / Self::USER_FEE_RATE_SCALE,
        })
    }

    pub(crate) async fn manage_agent<F>(
        &self,
        sender_address: &str,
        secure_preferences: Arc<dyn primitives::Preferences>,
        get_agents: F,
    ) -> Result<(bool, String, String), Box<dyn Error + Send + Sync>>
    where
        F: Future<Output = Result<Vec<AgentSession>, Box<dyn Error + Send + Sync>>>,
    {
        let agent = crate::agent::Agent::new(secure_preferences);
        let (agent_address, agent_private_key) = agent.get_or_create_credentials(sender_address)?;
        self.preferences.set(agent_owner_cache_key(&agent_address), sender_address.to_lowercase())?;
        let cache_key = self.cache_key(&agent_address, Self::AGENT_VALID_UNTIL_KEY);
        let current_time = Self::current_time()?;

        if let Some(cached_valid_until) = self.preferences.get_i64(&cache_key)?
            && current_time < cached_valid_until
        {
            return Ok((false, agent_address, agent_private_key));
        }

        let agents = get_agents.await?;

        if let Some(api_agent) = agents.iter().find(|a| a.address.to_lowercase() == agent_address.to_lowercase()) {
            let valid_until = (api_agent.valid_until / 1000) as i64;
            self.preferences.set_i64(&cache_key, valid_until)?;

            if current_time >= valid_until {
                let (new_address, new_key) = agent.regenerate_credentials(sender_address)?;
                self.preferences.set(agent_owner_cache_key(&new_address), sender_address.to_lowercase())?;
                Ok((true, new_address, new_key))
            } else {
                Ok((false, agent_address, agent_private_key))
            }
        } else {
            Ok((true, agent_address, agent_private_key))
        }
    }
}
