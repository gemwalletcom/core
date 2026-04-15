use crate::database::referrals::{ReferralUpdate, ReferralsStore};
use crate::database::rewards::{RewardsFilter, RewardsStore, RewardsUpdate};
use crate::database::transactions::{TransactionFilter, TransactionsStore};
use crate::database::usernames::{UsernameLookup, UsernamesStore};
use crate::database::wallets::WalletsStore;
use crate::models::{
    NewRewardEventRow, NewRewardReferralRow, NewRewardsRow, NewUsernameRow, ReferralAttemptRow, RewardEventRow, RewardReferralRow, RewardsRow, UsernameRow, WalletRow,
};
use crate::repositories::config_repository::ConfigRepository;
use crate::repositories::rewards_redemptions_repository::RewardsRedemptionsRepository;
use crate::sql_types::ChainRow;
use crate::sql_types::{RewardEventType, RewardRedemptionType, RewardStatus, TransactionState, UsernameStatus};
use crate::{DatabaseClient, DatabaseError, DieselResultExt, ReferralValidationError};
use chrono::NaiveDateTime;
use primitives::rewards::RewardStatus as PrimitiveRewardStatus;
use primitives::{Chain, ConfigKey, Device, NaiveDateTimeExt, ReferralLeader, ReferralLeaderboard, RewardEvent, Rewards, WalletId, now};

fn create_username_and_rewards(client: &mut DatabaseClient, wallet_id: i32, address: &str, device_id: i32) -> Result<RewardsRow, DatabaseError> {
    UsernamesStore::create_username(
        client,
        NewUsernameRow {
            username: address.to_string(),
            wallet_id,
            status: UsernameStatus::Unverified,
        },
    )?;
    Ok(RewardsStore::create_rewards(client, NewRewardsRow::new(address.to_string(), device_id))?)
}

fn validate_username(username: &str) -> Result<(), DatabaseError> {
    let len = username.len();
    if len < 4 {
        return Err(DatabaseError::Error("Username must be at least 4 characters".into()));
    }
    if len > 16 {
        return Err(DatabaseError::Error("Username must be at most 16 characters".into()));
    }
    if !username.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(DatabaseError::Error("Username must contain only letters and digits".into()));
    }
    Ok(())
}

fn find_username(client: &mut DatabaseClient, lookup: UsernameLookup<'_>) -> Result<Option<UsernameRow>, DatabaseError> {
    match UsernamesStore::get_username(client, lookup) {
        Ok(username) => Ok(Some(username)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn require_username(client: &mut DatabaseClient, lookup: UsernameLookup<'_>) -> Result<UsernameRow, DatabaseError> {
    match lookup {
        UsernameLookup::Username(username) => UsernamesStore::get_username(client, lookup).or_not_found(username.to_string()),
        UsernameLookup::WalletId(wallet_id) => UsernamesStore::get_username(client, lookup).or_not_found_internal(wallet_id.to_string()),
    }
}

fn require_rewards(client: &mut DatabaseClient, username: &str) -> Result<RewardsRow, DatabaseError> {
    RewardsStore::get_rewards_by_filter(client, vec![RewardsFilter::Username(username.to_string())])?
        .into_iter()
        .next()
        .ok_or_else(|| DatabaseError::not_found("Rewards", username.to_string()))
}

fn require_reward_event(client: &mut DatabaseClient, event_id: i32) -> Result<RewardEventRow, DatabaseError> {
    RewardsStore::get_event(client, event_id).or_not_found_internal(event_id.to_string())
}

fn find_wallet(client: &mut DatabaseClient, identifier: &str) -> Result<Option<WalletRow>, DatabaseError> {
    match WalletsStore::get_wallet(client, identifier) {
        Ok(wallet) => Ok(Some(wallet)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn require_wallet_by_id(client: &mut DatabaseClient, wallet_id: i32) -> Result<WalletRow, DatabaseError> {
    WalletsStore::get_wallet_by_id(client, wallet_id).or_not_found_internal(wallet_id.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewardsEligibilityConfig {
    pub activity_cutoff: NaiveDateTime,
    pub transactions_required: i64,
}

fn compute_verification_delay(base_delay: std::time::Duration, multiplier: i64, referrer_status: &PrimitiveRewardStatus) -> Option<std::time::Duration> {
    if referrer_status == &PrimitiveRewardStatus::Trusted || multiplier <= 0 {
        return None;
    }
    Some(std::time::Duration::from_secs(base_delay.as_secs() / multiplier as u64))
}

fn referral_verification_delay(config: &mut dyn ConfigRepository, referrer_status: &PrimitiveRewardStatus) -> Result<Option<std::time::Duration>, DatabaseError> {
    let base_delay = config.get_config_duration(ConfigKey::ReferralVerificationDelay)?;
    let multiplier = if referrer_status.is_verified() {
        config.get_config_i64(ConfigKey::ReferralVerifiedMultiplier)?
    } else {
        1
    };
    Ok(compute_verification_delay(base_delay, multiplier, referrer_status))
}

fn can_verify_referral(status: &PrimitiveRewardStatus, verify_after: Option<NaiveDateTime>) -> bool {
    if status.is_verified() {
        return true;
    }
    verify_after.is_some_and(|dt| dt <= now())
}

fn is_matching_pending_referral_confirmation(referral: &RewardReferralRow, referrer_username: &str, referred_username: &str) -> bool {
    referral.verified_at.is_none() && referral.referrer_username == referrer_username && referral.referred_username == referred_username
}

fn latest_wallet_device_id(client: &mut DatabaseClient, wallet_id: i32) -> Result<i32, DatabaseError> {
    WalletsStore::get_devices_by_wallet_id(client, wallet_id)?
        .into_iter()
        .max_by_key(|device| device.updated_at)
        .map(|device| device.id)
        .ok_or_else(|| DatabaseError::Error(format!("Wallet {wallet_id} has no subscribed devices")))
}

fn referred_username(client: &mut DatabaseClient, wallet_id: i32) -> Result<String, DatabaseError> {
    match find_username(client, UsernameLookup::WalletId(wallet_id))? {
        Some(username) => Ok(username.username),
        None => {
            let wallet = require_wallet_by_id(client, wallet_id)?;
            Ok(wallet.wallet_id.address().to_string())
        }
    }
}

fn ensure_wallet_reward_identity(client: &mut DatabaseClient, wallet_id: i32) -> Result<UsernameRow, DatabaseError> {
    let device_id = latest_wallet_device_id(client, wallet_id)?;

    match find_username(client, UsernameLookup::WalletId(wallet_id))? {
        Some(username) => {
            if require_rewards(client, &username.username).is_err() {
                RewardsStore::create_rewards(client, NewRewardsRow::new(username.username.clone(), device_id))?;
            }
            Ok(username)
        }
        None => {
            let wallet = require_wallet_by_id(client, wallet_id)?;
            let address = wallet.wallet_id.address().to_string();
            create_username_and_rewards(client, wallet_id, &address, device_id)?;
            require_username(client, UsernameLookup::WalletId(wallet_id))
        }
    }
}

fn add_referral_verified_event_rows(client: &mut DatabaseClient, referrer_username: &str, referred_username: &str) -> Result<Vec<RewardEventRow>, DatabaseError> {
    let referrer_event = RewardsStore::add_event(
        client,
        NewRewardEventRow {
            username: referrer_username.to_string(),
            event_type: RewardEventType::InviteNew,
        },
        RewardEventType::InviteNew.points(),
    )?;

    let referred_event = RewardsStore::add_event(
        client,
        NewRewardEventRow {
            username: referred_username.to_string(),
            event_type: RewardEventType::Joined,
        },
        RewardEventType::Joined.points(),
    )?;

    Ok(vec![referrer_event, referred_event])
}

fn add_referral_verified_events(client: &mut DatabaseClient, referrer_username: &str, referred_username: &str) -> Result<Vec<RewardEvent>, DatabaseError> {
    Ok(add_referral_verified_event_rows(client, referrer_username, referred_username)?
        .into_iter()
        .map(|event| event.as_primitive())
        .collect())
}

fn add_referral_pending_events(client: &mut DatabaseClient, referrer_username: &str) -> Result<Vec<RewardEvent>, DatabaseError> {
    let event = RewardsStore::add_event(
        client,
        NewRewardEventRow {
            username: referrer_username.to_string(),
            event_type: RewardEventType::InvitePending,
        },
        RewardEventType::InvitePending.points(),
    )?;
    Ok(vec![event.as_primitive()])
}

fn add_referral_with_events(
    client: &mut DatabaseClient,
    referrer_username: &str,
    referred_username: &str,
    device_id: i32,
    risk_signal_id: i32,
    verified_at: Option<NaiveDateTime>,
) -> Result<Vec<RewardEvent>, DatabaseError> {
    ReferralsStore::add_referral(
        client,
        NewRewardReferralRow {
            referrer_username: referrer_username.to_string(),
            referred_username: referred_username.to_string(),
            referred_device_id: device_id,
            risk_signal_id,
            verified_at,
        },
    )?;

    if verified_at.is_some() {
        add_referral_verified_events(client, referrer_username, referred_username)
    } else {
        add_referral_pending_events(client, referrer_username)
    }
}

fn complete_referral(client: &mut DatabaseClient, referred_username: &str) -> Result<Vec<i32>, DatabaseError> {
    let Some(referral) = ReferralsStore::get_referral_by_username(client, referred_username)? else {
        return Ok(vec![]);
    };

    if referral.verified_at.is_some() {
        return Ok(vec![]);
    }

    ReferralsStore::update_referral(client, referral.id, ReferralUpdate::VerifiedAt(now()))?;
    Ok(add_referral_verified_event_rows(client, &referral.referrer_username, referred_username)?
        .into_iter()
        .map(|event| event.id)
        .collect())
}

pub trait RewardsRepository {
    fn get_reward_by_wallet_id(&mut self, wallet_id: i32) -> Result<Rewards, DatabaseError>;
    fn get_reward_events_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<RewardEvent>, DatabaseError>;
    fn get_reward_event(&mut self, event_id: i32) -> Result<RewardEvent, DatabaseError>;
    fn get_reward_event_devices(&mut self, event_id: i32) -> Result<Vec<Device>, DatabaseError>;
    fn create_reward(&mut self, wallet_id: i32, username: &str) -> Result<(Rewards, i32), DatabaseError>;
    fn change_username(&mut self, wallet_id: i32, new_username: &str) -> Result<Rewards, DatabaseError>;
    fn get_referral_code(&mut self, code: &str) -> Result<Option<String>, DatabaseError>;
    fn validate_referral_use(&mut self, referrer_username: &str, wallet_id: i32, device_id: i32, eligibility_days: i64) -> Result<(), ReferralValidationError>;
    fn add_referral_attempt(&mut self, referrer_username: &str, referred_wallet_id: i32, device_id: i32, risk_signal_id: Option<i32>, reason: &str) -> Result<(), DatabaseError>;
    fn get_first_subscription_date_by_wallet_id(&mut self, wallet_id: i32) -> Result<Option<NaiveDateTime>, DatabaseError>;
    fn get_wallet_id_by_username(&mut self, username: &str) -> Result<i32, DatabaseError>;
    fn get_referrer_username(&mut self, referred_username: &str) -> Result<Option<String>, DatabaseError>;
    fn get_address_by_username(&mut self, username: &str) -> Result<String, DatabaseError>;
    fn get_status_by_username(&mut self, username: &str) -> Result<PrimitiveRewardStatus, DatabaseError>;
    fn get_referral_count_by_username(&mut self, username: &str) -> Result<i32, DatabaseError>;
    fn count_referrals_since(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn get_rewards_leaderboard(&mut self) -> Result<ReferralLeaderboard, DatabaseError>;
    fn disable_rewards(&mut self, username: &str, reason: &str, comment: &str) -> Result<i32, DatabaseError>;
    fn get_rewards_by_filter(&mut self, filters: Vec<RewardsFilter>) -> Result<Vec<RewardsRow>, DatabaseError>;
    fn check_eligibility(&mut self, username: &str, eligibility: RewardsEligibilityConfig) -> Result<Option<i32>, DatabaseError>;
    fn promote_to_verified(&mut self, username: &str) -> Result<Vec<i32>, DatabaseError>;

    fn use_or_verify_referral(&mut self, referrer_username: &str, referred_wallet_id: i32, device_id: i32, risk_signal_id: i32) -> Result<Vec<RewardEvent>, DatabaseError>;
}

impl RewardsRepository for DatabaseClient {
    fn get_reward_by_wallet_id(&mut self, wallet_id: i32) -> Result<Rewards, DatabaseError> {
        let username = ensure_wallet_reward_identity(self, wallet_id)?;
        let rewards = require_rewards(self, &username.username)?;
        let has_custom_code = username.has_custom_username();
        let code = if has_custom_code { Some(username.username.clone()) } else { None };

        let status = *rewards.status;
        let types = [RewardRedemptionType::Asset];
        let options = RewardsRedemptionsRepository::get_redemption_options(self, &types)?
            .into_iter()
            .filter(|x| x.remaining.unwrap_or_default() > 0)
            .collect();

        Ok(Rewards {
            code,
            invite_reward_points: RewardEventType::InviteNew.points(),
            referral_count: rewards.referral_count,
            points: rewards.points,
            used_referral_code: rewards.referrer_username,
            status,
            created_at: rewards.created_at,
            verify_after: rewards.verify_after.map(|dt| dt.and_utc()),
            redemption_options: options,
            disable_reason: rewards.disable_reason.clone(),
            referral_allowance: Default::default(),
        })
    }

    fn get_reward_events_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<RewardEvent>, DatabaseError> {
        let username = ensure_wallet_reward_identity(self, wallet_id)?;
        let events = RewardsStore::get_events(self, &username.username)?;
        Ok(events.iter().map(|e| e.as_primitive()).collect())
    }

    fn get_reward_event(&mut self, event_id: i32) -> Result<RewardEvent, DatabaseError> {
        let event = require_reward_event(self, event_id)?;
        Ok(event.as_primitive())
    }

    fn get_reward_event_devices(&mut self, event_id: i32) -> Result<Vec<Device>, DatabaseError> {
        let event = require_reward_event(self, event_id)?;
        let username = require_username(self, UsernameLookup::Username(&event.username))?;
        let devices = WalletsStore::get_devices_by_wallet_id(self, username.wallet_id)?;
        Ok(devices.into_iter().map(|d| d.as_primitive()).collect())
    }

    fn create_reward(&mut self, wallet_id: i32, username: &str) -> Result<(Rewards, i32), DatabaseError> {
        validate_username(username)?;

        if find_username(self, UsernameLookup::Username(username))?.is_some() {
            return Err(DatabaseError::Error("Username already taken".into()));
        }

        let existing = ensure_wallet_reward_identity(self, wallet_id)?;
        if existing.has_custom_username() {
            return Err(DatabaseError::Error("Wallet already has a username".into()));
        }

        let existing_rewards = require_rewards(self, &existing.username)?;
        if existing_rewards.status.is_disabled() {
            return Err(DatabaseError::Error("Rewards are not enabled for this user".into()));
        }

        UsernamesStore::update_username(self, wallet_id, username).or_not_found_internal(wallet_id.to_string())?;

        let event = RewardsStore::add_event(
            self,
            NewRewardEventRow {
                username: username.to_string(),
                event_type: RewardEventType::CreateUsername,
            },
            RewardEventType::CreateUsername.points(),
        )?;

        let rewards = self.get_reward_by_wallet_id(wallet_id)?;
        Ok((rewards, event.id))
    }

    fn change_username(&mut self, wallet_id: i32, new_username: &str) -> Result<Rewards, DatabaseError> {
        validate_username(new_username)?;

        let existing = require_username(self, UsernameLookup::WalletId(wallet_id))?;

        if !existing.has_custom_username() {
            return Err(DatabaseError::Error("No custom username to change".into()));
        }

        if existing.username.eq_ignore_ascii_case(new_username) {
            return Err(DatabaseError::Error("New username is the same as current".into()));
        }

        if find_username(self, UsernameLookup::Username(new_username))?.is_some() {
            return Err(DatabaseError::Error("Username already taken".into()));
        }

        let rewards = require_rewards(self, &existing.username)?;
        if rewards.status.is_disabled() {
            return Err(DatabaseError::Error("Rewards are not enabled for this user".into()));
        }

        UsernamesStore::change_username(self, &existing.username, new_username)?;

        self.get_reward_by_wallet_id(wallet_id)
    }

    fn get_referral_code(&mut self, code: &str) -> Result<Option<String>, DatabaseError> {
        Ok(find_username(self, UsernameLookup::Username(code))?.map(|username| username.username))
    }

    fn validate_referral_use(&mut self, referrer_username: &str, wallet_id: i32, device_id: i32, eligibility_days: i64) -> Result<(), ReferralValidationError> {
        let referrer = require_username(self, UsernameLookup::Username(referrer_username))?;
        let referrer_rewards = require_rewards(self, referrer_username)?;

        if !referrer_rewards.status.is_verified() {
            return Err(ReferralValidationError::RewardsNotEnabled(referrer_username.to_string()));
        }

        let device_subscriptions = WalletsStore::get_device_addresses(self, device_id, ChainRow::from(Chain::Ethereum))?;

        for address in &device_subscriptions {
            let wallet_identifier = WalletId::Multicoin(address.clone()).id();
            if let Some(wallet) = find_wallet(self, &wallet_identifier)? {
                if let Some(first_subscription_at) = WalletsStore::get_first_subscription_date_by_wallet_id(self, wallet.id)?
                    && first_subscription_at.is_older_than_days(eligibility_days)
                {
                    return Err(ReferralValidationError::EligibilityExpired(eligibility_days));
                }
                if referrer.wallet_id == wallet.id {
                    return Err(ReferralValidationError::CannotReferSelf);
                }
            }
        }

        if let Some(referral) = ReferralsStore::get_referral_by_referred_device_id(self, device_id)? {
            let referred_username = referred_username(self, wallet_id)?;
            if !is_matching_pending_referral_confirmation(&referral, referrer_username, &referred_username) {
                return Err(ReferralValidationError::DeviceAlreadyUsed);
            }
        }

        Ok(())
    }

    fn add_referral_attempt(&mut self, referrer_username: &str, wallet_id: i32, device_id: i32, risk_signal_id: Option<i32>, reason: &str) -> Result<(), DatabaseError> {
        ReferralsStore::add_referral_attempt(
            self,
            ReferralAttemptRow {
                referrer_username: referrer_username.to_string(),
                wallet_id,
                device_id,
                risk_signal_id,
                reason: reason.to_string(),
            },
        )?;
        Ok(())
    }

    fn get_first_subscription_date_by_wallet_id(&mut self, wallet_id: i32) -> Result<Option<NaiveDateTime>, DatabaseError> {
        Ok(WalletsStore::get_first_subscription_date_by_wallet_id(self, wallet_id)?)
    }

    fn get_wallet_id_by_username(&mut self, username: &str) -> Result<i32, DatabaseError> {
        let username = require_username(self, UsernameLookup::Username(username))?;
        Ok(username.wallet_id)
    }

    fn get_referrer_username(&mut self, referred_username: &str) -> Result<Option<String>, DatabaseError> {
        let referral = ReferralsStore::get_referral_by_username(self, referred_username)?;
        Ok(referral.map(|r| r.referrer_username))
    }

    fn get_address_by_username(&mut self, username: &str) -> Result<String, DatabaseError> {
        let username_row = require_username(self, UsernameLookup::Username(username))?;
        let wallet = require_wallet_by_id(self, username_row.wallet_id)?;
        Ok(wallet.wallet_id.address().to_string())
    }

    fn get_status_by_username(&mut self, username: &str) -> Result<PrimitiveRewardStatus, DatabaseError> {
        let rewards = require_rewards(self, username)?;
        Ok(*rewards.status)
    }

    fn get_referral_count_by_username(&mut self, username: &str) -> Result<i32, DatabaseError> {
        let rewards = require_rewards(self, username)?;
        Ok(rewards.referral_count)
    }

    fn count_referrals_since(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(ReferralsStore::count_referrals_since(self, referrer_username, since)?)
    }

    fn get_rewards_leaderboard(&mut self) -> Result<ReferralLeaderboard, DatabaseError> {
        let current = now();
        let limit = 10;
        let invite_types = [RewardEventType::InviteNew];
        let points_per_referral = RewardEventType::InviteNew.points() as i64;

        let map_entry = |(username, referrals): (String, i64)| ReferralLeader {
            username,
            referrals: referrals as i32,
            points: (referrals * points_per_referral) as i32,
        };

        let daily = RewardsStore::get_top_referrers_since(self, &invite_types, current.days_ago(1), limit)?
            .into_iter()
            .map(map_entry)
            .collect();

        let weekly = RewardsStore::get_top_referrers_since(self, &invite_types, current.days_ago(7), limit)?
            .into_iter()
            .map(map_entry)
            .collect();

        let monthly = RewardsStore::get_top_referrers_since(self, &invite_types, current.days_ago(30), limit)?
            .into_iter()
            .map(map_entry)
            .collect();

        Ok(ReferralLeaderboard { daily, weekly, monthly })
    }

    fn disable_rewards(&mut self, username: &str, reason: &str, comment: &str) -> Result<i32, DatabaseError> {
        Ok(RewardsStore::disable_rewards(self, username, reason, comment)?)
    }

    fn get_rewards_by_filter(&mut self, filters: Vec<RewardsFilter>) -> Result<Vec<RewardsRow>, DatabaseError> {
        Ok(RewardsStore::get_rewards_by_filter(self, filters)?)
    }

    fn check_eligibility(&mut self, username: &str, eligibility: RewardsEligibilityConfig) -> Result<Option<i32>, DatabaseError> {
        let username_row = require_username(self, UsernameLookup::Username(username))?;
        let rewards = require_rewards(self, &username_row.username)?;

        if *rewards.status != PrimitiveRewardStatus::Unverified {
            return Ok(None);
        }

        if rewards.verify_after.is_some_and(|dt| dt > now()) {
            return Ok(None);
        }

        let Some(first_subscription_at) = WalletsStore::get_first_subscription_date_by_wallet_id(self, username_row.wallet_id)? else {
            return Ok(None);
        };

        if first_subscription_at > eligibility.activity_cutoff {
            return Ok(None);
        }

        let Some(latest_activity_at) = WalletsStore::get_devices_by_wallet_id(self, username_row.wallet_id)?
            .into_iter()
            .map(|device| device.updated_at)
            .max()
        else {
            return Ok(None);
        };

        if latest_activity_at < eligibility.activity_cutoff {
            return Ok(None);
        }

        let transactions_current = TransactionsStore::get_transactions_by_wallet_since(
            self,
            username_row.wallet_id,
            first_subscription_at,
            vec![TransactionFilter::States(vec![TransactionState::Confirmed])],
        )?
        .len() as i64;

        if transactions_current < eligibility.transactions_required {
            return Ok(None);
        }

        Ok(Some(username_row.wallet_id))
    }

    fn promote_to_verified(&mut self, username: &str) -> Result<Vec<i32>, DatabaseError> {
        RewardsStore::update_rewards(self, username, RewardsUpdate::Status(RewardStatus::Verified))?;
        complete_referral(self, username)
    }

    fn use_or_verify_referral(&mut self, referrer_username: &str, referred_wallet_id: i32, device_id: i32, risk_signal_id: i32) -> Result<Vec<RewardEvent>, DatabaseError> {
        let referred_username = ensure_wallet_reward_identity(self, referred_wallet_id)?.username;
        let referred_rewards = require_rewards(self, &referred_username)?;
        let referrer_rewards = require_rewards(self, referrer_username)?;
        let can_verify = can_verify_referral(&referred_rewards.status, referred_rewards.verify_after);

        if can_verify && !referred_rewards.status.is_verified() {
            RewardsStore::update_rewards(self, &referred_username, RewardsUpdate::Status(RewardStatus::Unverified))?;
            RewardsStore::update_rewards(self, &referred_username, RewardsUpdate::ClearVerifyAfter)?;
        }

        match ReferralsStore::get_referral_by_username(self, &referred_username)? {
            Some(referral) if referral.verified_at.is_none() => self.confirm_pending_referral(referral, referrer_username, &referred_username, device_id, can_verify),
            Some(_) => Err(DatabaseError::Error("Referral already verified".to_string())),
            None => self.create_new_referral(referrer_username, &referred_username, device_id, risk_signal_id, can_verify, &referrer_rewards.status),
        }
    }
}

impl DatabaseClient {
    fn confirm_pending_referral(
        &mut self,
        referral: RewardReferralRow,
        referrer_username: &str,
        referred_username: &str,
        device_id: i32,
        can_verify: bool,
    ) -> Result<Vec<RewardEvent>, DatabaseError> {
        if referral.referrer_username != referrer_username {
            return Err(DatabaseError::Error("Referral code does not match pending referral".to_string()));
        }
        if referral.referred_device_id != device_id {
            return Err(DatabaseError::Error("Must verify from same device".to_string()));
        }
        if can_verify {
            ReferralsStore::update_referral(self, referral.id, ReferralUpdate::VerifiedAt(now()))?;
            add_referral_verified_events(self, &referral.referrer_username, referred_username)
        } else {
            Ok(vec![])
        }
    }

    fn create_new_referral(
        &mut self,
        referrer_username: &str,
        referred_username: &str,
        device_id: i32,
        risk_signal_id: i32,
        can_verify: bool,
        referrer_status: &PrimitiveRewardStatus,
    ) -> Result<Vec<RewardEvent>, DatabaseError> {
        let delay = referral_verification_delay(self.config(), referrer_status)?;

        if !can_verify && let Some(delay) = delay {
            let verify_after = now() + chrono::Duration::seconds(delay.as_secs() as i64);
            RewardsStore::update_rewards(self, referred_username, RewardsUpdate::VerifyAfter(verify_after))?;
            RewardsStore::update_rewards(self, referred_username, RewardsUpdate::Status(RewardStatus::Pending))?;
        }

        let skip_delay = can_verify || delay.is_none();
        let verified_at = skip_delay.then_some(now());
        add_referral_with_events(self, referrer_username, referred_username, device_id, risk_signal_id, verified_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UsernameRow;

    fn username_row(username: &str, wallet_id: i32) -> UsernameRow {
        UsernameRow {
            username: username.to_string(),
            wallet_id,
            status: UsernameStatus::Unverified,
        }
    }

    #[test]
    fn test_has_custom_username() {
        assert!(username_row("alice", 1).has_custom_username());
        assert!(username_row("user1234", 1).has_custom_username());
        assert!(!username_row("0x1234567890abcdef1234567890abcdef12345678", 1).has_custom_username());
        assert!(!username_row("wallet_1", 1).has_custom_username());
    }

    #[test]
    fn test_validate_username() {
        assert!(validate_username("abcd").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("1234567890123456").is_ok());

        assert!(validate_username("abc").is_err());
        assert!(validate_username("12345678901234567").is_err());
        assert!(validate_username("user_name").is_err());
        assert!(validate_username("user-name").is_err());
        assert!(validate_username("user.name").is_err());
        assert!(validate_username("user name").is_err());
    }

    #[test]
    fn test_can_verify_referral() {
        assert!(can_verify_referral(&PrimitiveRewardStatus::Verified, None));
        assert!(can_verify_referral(&PrimitiveRewardStatus::Trusted, None));

        assert!(!can_verify_referral(&PrimitiveRewardStatus::Unverified, None));
        assert!(!can_verify_referral(&PrimitiveRewardStatus::Pending, None));

        let past = (chrono::Utc::now() - chrono::Duration::hours(1)).naive_utc();
        assert!(can_verify_referral(&PrimitiveRewardStatus::Unverified, Some(past)));
        assert!(can_verify_referral(&PrimitiveRewardStatus::Pending, Some(past)));

        let future = (chrono::Utc::now() + chrono::Duration::hours(1)).naive_utc();
        assert!(!can_verify_referral(&PrimitiveRewardStatus::Unverified, Some(future)));
        assert!(!can_verify_referral(&PrimitiveRewardStatus::Pending, Some(future)));

        assert!(can_verify_referral(&RewardStatus::Verified, Some(future)));
    }

    #[test]
    fn test_is_matching_pending_referral_confirmation() {
        let now = chrono::Utc::now().naive_utc();
        let referral = RewardReferralRow {
            id: 1,
            referrer_username: "alice".to_string(),
            referred_username: "bob".to_string(),
            referred_device_id: 10,
            risk_signal_id: 20,
            verified_at: None,
            updated_at: now,
            created_at: now,
        };

        assert!(is_matching_pending_referral_confirmation(&referral, "alice", "bob"));
        assert!(!is_matching_pending_referral_confirmation(&referral, "charlie", "bob"));
        assert!(!is_matching_pending_referral_confirmation(&referral, "alice", "dave"));

        let verified_referral = RewardReferralRow {
            verified_at: Some(now),
            ..referral
        };
        assert!(!is_matching_pending_referral_confirmation(&verified_referral, "alice", "bob"));
    }

    #[test]
    fn test_compute_verification_delay() {
        let base = std::time::Duration::from_secs(86400); // 24h

        // Trusted: no delay regardless of multiplier
        assert_eq!(compute_verification_delay(base, 2, &PrimitiveRewardStatus::Trusted), None);

        // Verified with multiplier 2: 24h / 2 = 12h
        assert_eq!(compute_verification_delay(base, 2, &PrimitiveRewardStatus::Verified), Some(std::time::Duration::from_secs(43200)));

        // Unverified: full delay (multiplier applied as 1 by caller)
        assert_eq!(compute_verification_delay(base, 1, &PrimitiveRewardStatus::Unverified), Some(base));

        // Multiplier 0: no delay
        assert_eq!(compute_verification_delay(base, 0, &PrimitiveRewardStatus::Verified), None);
    }
}
