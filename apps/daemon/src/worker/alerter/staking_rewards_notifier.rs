use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cacher::CacheKey;
use cacher::CacherClient;
use gem_tracing::info_with_fields;
use localizer::LanguageLocalizer;
use num_bigint::BigUint;
use number_formatter::{BigNumberFormatter, ValueFormatter, ValueStyle};
use primitives::{Asset, Chain, DelegationBase, DeviceSubscription, GorushNotification, PushNotification, TransactionType};
use settings_chain::ChainProviders;
use storage::{Database, TransactionsRepository, WalletsRepository};
use streamer::{NotificationsPayload, StreamProducer, StreamProducerQueue};

#[derive(Clone, Copy)]
pub struct StakeRewardsConfig {
    pub threshold: f64,
    pub lookback: Duration,
}

pub struct StakingRewardsNotifier {
    chain_providers: Arc<ChainProviders>,
    database: Database,
    config: StakeRewardsConfig,
    cacher: CacherClient,
    stream_producer: StreamProducer,
}

impl StakingRewardsNotifier {
    pub fn new(chain_providers: Arc<ChainProviders>, database: Database, config: StakeRewardsConfig, cacher: CacherClient, stream_producer: StreamProducer) -> Self {
        Self {
            chain_providers,
            database,
            config,
            cacher,
            stream_producer,
        }
    }

    pub async fn check_chain(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let since = chrono::Utc::now().naive_utc() - chrono::Duration::from_std(self.config.lookback)?;
        let kinds = TransactionType::staking_types().into_iter().map(Into::into).collect();
        let addresses = self.database.transactions()?.get_addresses_by_chain_and_kind(chain.as_ref(), kinds, since)?;

        let mut notified = 0;
        for address in &addresses {
            match self.process_address(chain, address).await {
                Ok(true) => notified += 1,
                Ok(false) => {}
                Err(e) => {
                    gem_tracing::error("staking rewards notifier", e.as_ref());
                }
            }
        }

        info_with_fields!("staking rewards notifier", chain = chain.as_ref(), addresses = addresses.len(), notified = notified);
        Ok(notified)
    }

    async fn process_address(&self, chain: Chain, address: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.wallets()?.get_subscriptions_by_chain_addresses(chain, vec![address.to_string()])?;
        if subscriptions.is_empty() {
            return Ok(false);
        }

        if !self.cacher.can_process_cached(CacheKey::AlerterStakeRewards(chain.as_ref(), address)).await? {
            return Ok(false);
        }

        let delegations = self.chain_providers.get_staking_delegations(chain, address.to_string()).await?;

        let total_staked = DelegationBase::total_active_balance(&delegations);
        let total_rewards = DelegationBase::total_active_rewards(&delegations);
        if total_staked == BigUint::from(0u32) || total_rewards == BigUint::from(0u32) {
            return Ok(false);
        }

        if BigNumberFormatter::ratio(&total_rewards, &total_staked) < self.config.threshold {
            return Ok(false);
        }

        let asset = Asset::from_chain(chain);
        let rewards_value = ValueFormatter::format(ValueStyle::Auto, &total_rewards.to_string(), asset.decimals)?;

        let notifications: Vec<_> = subscriptions
            .into_iter()
            .filter_map(|sub| Self::create_notification(sub, &rewards_value, &asset))
            .collect();

        self.stream_producer.publish_notifications_observers(NotificationsPayload::new(notifications)).await?;

        Ok(true)
    }

    fn create_notification(sub: DeviceSubscription, rewards_value: &str, asset: &Asset) -> Option<GorushNotification> {
        let localizer = LanguageLocalizer::new_with_language(&sub.device.locale);
        let notification = localizer.notification_stake_rewards(rewards_value, &asset.name);
        let push = PushNotification::new_stake(sub.wallet_id, asset.id.clone());
        GorushNotification::from_device(sub.device, notification.title, notification.description, push)
    }
}
