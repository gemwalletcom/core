use crate::database::rewards::{ReferralUpdate, RewardsStore};
use crate::database::usernames::{UsernameLookup, UsernamesStore};
use crate::database::wallets::WalletsStore;
use crate::models::{NewRewardEventRow, NewRewardReferralRow, NewRewardsRow, NewUsernameRow, ReferralAttemptRow, RewardEventRow, RewardsRow, UsernameRow, WalletRow};
use crate::repositories::rewards_redemptions_repository::RewardsRedemptionsRepository;
use crate::sql_types::ChainRow;
use crate::sql_types::{RewardEventType, RewardRedemptionType, RewardStatus, UsernameStatus};
use crate::{DatabaseClient, DatabaseError, DieselResultExt, ReferralValidationError};
use chrono::Duration as ChronoDuration;
use chrono::NaiveDateTime;
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
    RewardsStore::get_rewards(client, username).or_not_found(username.to_string())
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

pub trait RewardsRepository {
    fn get_reward_by_wallet_id(&mut self, wallet_id: i32) -> Result<Rewards, DatabaseError>;
    fn get_reward_events_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<RewardEvent>, DatabaseError>;
    fn get_reward_event(&mut self, event_id: i32) -> Result<RewardEvent, DatabaseError>;
    fn get_reward_event_devices(&mut self, event_id: i32) -> Result<Vec<Device>, DatabaseError>;
    fn create_reward(&mut self, wallet_id: i32, username: &str, device_id: i32) -> Result<(Rewards, i32), DatabaseError>;
    fn change_username(&mut self, wallet_id: i32, new_username: &str) -> Result<Rewards, DatabaseError>;
    fn get_referral_code(&mut self, code: &str) -> Result<Option<String>, DatabaseError>;
    fn validate_referral_use(&mut self, referrer_username: &str, device_id: i32, eligibility_days: i64) -> Result<(), ReferralValidationError>;
    fn add_referral_attempt(&mut self, referrer_username: &str, referred_wallet_id: i32, device_id: i32, risk_signal_id: Option<i32>, reason: &str) -> Result<(), DatabaseError>;
    fn get_first_subscription_date(&mut self, addresses: Vec<String>) -> Result<Option<NaiveDateTime>, DatabaseError>;
    fn get_wallet_id_by_username(&mut self, username: &str) -> Result<i32, DatabaseError>;
    fn get_referrer_username(&mut self, referred_username: &str) -> Result<Option<String>, DatabaseError>;
    fn get_address_by_username(&mut self, username: &str) -> Result<String, DatabaseError>;
    fn get_username_by_wallet_id(&mut self, wallet_id: i32) -> Result<Option<String>, DatabaseError>;
    fn is_verified_by_username(&mut self, username: &str) -> Result<bool, DatabaseError>;
    fn get_status_by_username(&mut self, username: &str) -> Result<primitives::rewards::RewardStatus, DatabaseError>;
    fn get_referral_count_by_username(&mut self, username: &str) -> Result<i32, DatabaseError>;
    fn count_referrals_since(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn get_rewards_leaderboard(&mut self) -> Result<ReferralLeaderboard, DatabaseError>;
    fn disable_rewards(&mut self, username: &str, reason: &str, comment: &str) -> Result<i32, DatabaseError>;

    fn use_or_verify_referral(&mut self, referrer_username: &str, referred_wallet_id: i32, device_id: i32, risk_signal_id: i32) -> Result<Vec<RewardEvent>, DatabaseError>;
}

impl RewardsRepository for DatabaseClient {
    fn get_reward_by_wallet_id(&mut self, wallet_id: i32) -> Result<Rewards, DatabaseError> {
        let username = require_username(self, UsernameLookup::WalletId(wallet_id))?;
        let rewards = require_rewards(self, &username.username)?;

        let has_custom_code = username.has_custom_username();
        let code = if has_custom_code { Some(username.username.clone()) } else { None };

        let status = *rewards.status;
        let options = if status.is_enabled() {
            let types = [RewardRedemptionType::Asset];
            RewardsRedemptionsRepository::get_redemption_options(self, &types)?
                .into_iter()
                .filter(|x| x.remaining.unwrap_or_default() > 0)
                .collect()
        } else {
            vec![]
        };

        Ok(Rewards {
            code,
            referral_count: rewards.referral_count,
            points: rewards.points,
            used_referral_code: rewards.referrer_username,
            status,
            is_enabled: status.is_enabled(),
            verified: status.is_verified(),
            created_at: rewards.created_at,
            verify_after: rewards.verify_after.map(|dt| dt.and_utc()),
            redemption_options: options,
            disable_reason: rewards.disable_reason.clone(),
            referral_allowance: Default::default(),
        })
    }

    fn get_reward_events_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<RewardEvent>, DatabaseError> {
        let username = require_username(self, UsernameLookup::WalletId(wallet_id))?;
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

    fn create_reward(&mut self, wallet_id: i32, username: &str, device_id: i32) -> Result<(Rewards, i32), DatabaseError> {
        validate_username(username)?;

        if find_username(self, UsernameLookup::Username(username))?.is_some() {
            return Err(DatabaseError::Error("Username already taken".into()));
        }

        if let Some(existing) = find_username(self, UsernameLookup::WalletId(wallet_id))? {
            if existing.has_custom_username() {
                return Err(DatabaseError::Error("Wallet already has a username".into()));
            }
            let existing_rewards = require_rewards(self, &existing.username)?;
            if !existing_rewards.status.is_enabled() {
                return Err(DatabaseError::Error("Rewards are not enabled for this user".into()));
            }
            UsernamesStore::update_username(self, wallet_id, username).or_not_found_internal(wallet_id.to_string())?;
        } else {
            UsernamesStore::create_username(
                self,
                NewUsernameRow {
                    username: username.to_string(),
                    wallet_id,
                    status: UsernameStatus::Unverified,
                },
            )?;
            RewardsStore::create_rewards(
                self,
                NewRewardsRow {
                    username: username.to_string(),
                    status: RewardStatus::Unverified,
                    level: None,
                    points: 0,
                    referrer_username: None,
                    referral_count: 0,
                    device_id,
                    comment: None,
                    disable_reason: None,
                    verify_after: None,
                },
            )?;
        }

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
        if !rewards.status.is_enabled() {
            return Err(DatabaseError::Error("Rewards are not enabled for this user".into()));
        }

        UsernamesStore::change_username(self, &existing.username, new_username)?;

        self.get_reward_by_wallet_id(wallet_id)
    }

    fn get_referral_code(&mut self, code: &str) -> Result<Option<String>, DatabaseError> {
        Ok(find_username(self, UsernameLookup::Username(code))?.map(|username| username.username))
    }

    fn validate_referral_use(&mut self, referrer_username: &str, device_id: i32, eligibility_days: i64) -> Result<(), ReferralValidationError> {
        let referrer = require_username(self, UsernameLookup::Username(referrer_username))?;
        let referrer_rewards = require_rewards(self, referrer_username)?;

        if !referrer_rewards.status.is_enabled() {
            return Err(ReferralValidationError::RewardsNotEnabled(referrer_username.to_string()));
        }

        let device_subscriptions = WalletsStore::get_device_addresses(self, device_id, ChainRow::from(Chain::Ethereum))?;

        for address in &device_subscriptions {
            let wallet_identifier = WalletId::Multicoin(address.clone()).id();
            if let Some(wallet) = find_wallet(self, &wallet_identifier)?
                && let Some(username) = find_username(self, UsernameLookup::WalletId(wallet.id))?
            {
                let rewards = require_rewards(self, &username.username)?;
                if rewards.created_at.is_older_than_days(eligibility_days) {
                    return Err(ReferralValidationError::EligibilityExpired(eligibility_days));
                }
                if referrer_username == username.username || referrer.wallet_id == username.wallet_id {
                    return Err(ReferralValidationError::CannotReferSelf);
                }
            }
        }

        if RewardsStore::get_referral_by_referred_device_id(self, device_id)?.is_some() {
            return Err(ReferralValidationError::DeviceAlreadyUsed);
        }

        Ok(())
    }

    fn add_referral_attempt(&mut self, referrer_username: &str, wallet_id: i32, device_id: i32, risk_signal_id: Option<i32>, reason: &str) -> Result<(), DatabaseError> {
        RewardsStore::add_referral_attempt(
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

    fn get_first_subscription_date(&mut self, addresses: Vec<String>) -> Result<Option<NaiveDateTime>, DatabaseError> {
        Ok(WalletsStore::get_first_subscription_date(self, addresses)?)
    }

    fn get_wallet_id_by_username(&mut self, username: &str) -> Result<i32, DatabaseError> {
        let username = require_username(self, UsernameLookup::Username(username))?;
        Ok(username.wallet_id)
    }

    fn get_referrer_username(&mut self, referred_username: &str) -> Result<Option<String>, DatabaseError> {
        let referral = RewardsStore::get_referral_by_username(self, referred_username)?;
        Ok(referral.map(|r| r.referrer_username))
    }

    fn get_address_by_username(&mut self, username: &str) -> Result<String, DatabaseError> {
        let username_row = require_username(self, UsernameLookup::Username(username))?;
        let wallet = require_wallet_by_id(self, username_row.wallet_id)?;
        Ok(wallet.wallet_id.address().to_string())
    }

    fn get_username_by_wallet_id(&mut self, wallet_id: i32) -> Result<Option<String>, DatabaseError> {
        Ok(find_username(self, UsernameLookup::WalletId(wallet_id))?.map(|username| username.username))
    }

    fn is_verified_by_username(&mut self, username: &str) -> Result<bool, DatabaseError> {
        let rewards = require_rewards(self, username)?;
        Ok(rewards.status.is_verified())
    }

    fn get_status_by_username(&mut self, username: &str) -> Result<primitives::rewards::RewardStatus, DatabaseError> {
        let rewards = require_rewards(self, username)?;
        Ok(*rewards.status)
    }

    fn get_referral_count_by_username(&mut self, username: &str) -> Result<i32, DatabaseError> {
        let rewards = require_rewards(self, username)?;
        Ok(rewards.referral_count)
    }

    fn count_referrals_since(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RewardsStore::count_referrals_since(self, referrer_username, since)?)
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

    fn use_or_verify_referral(&mut self, referrer_username: &str, referred_wallet_id: i32, device_id: i32, risk_signal_id: i32) -> Result<Vec<RewardEvent>, DatabaseError> {
        let verification_date = self.get_referral_verification_date(now())?;
        let verified_at = if verification_date.is_none() { Some(now()) } else { None };

        let referred_identifier = match find_username(self, UsernameLookup::WalletId(referred_wallet_id))? {
            Some(username) => username.username,
            None => {
                let wallet = require_wallet_by_id(self, referred_wallet_id)?;
                let address = wallet.wallet_id.address().to_string();
                create_username_and_rewards(self, referred_wallet_id, &address, device_id)?.username
            }
        };

        match RewardsStore::get_referral_by_username(self, &referred_identifier)? {
            Some(referral) if referral.verified_at.is_none() => {
                if referral.referrer_username != referrer_username {
                    return Err(DatabaseError::Error("Referral code does not match pending referral".to_string()));
                }

                if referral.referred_device_id != device_id {
                    return Err(DatabaseError::Error("Must verify from same device".to_string()));
                }

                RewardsStore::update_referral(self, referral.id, ReferralUpdate::VerifiedAt(now()))?;
                self.add_referral_verified_events(&referral.referrer_username, &referred_identifier)
            }
            Some(_) => Err(DatabaseError::Error("Referral already verified".to_string())),
            None => self.add_referral_with_events(referrer_username, &referred_identifier, device_id, risk_signal_id, verified_at),
        }
    }
}

impl DatabaseClient {
    fn get_referral_verification_date(&mut self, created_at: NaiveDateTime) -> Result<Option<chrono::DateTime<chrono::Utc>>, DatabaseError> {
        let delay = self.config().get_config_duration(ConfigKey::ReferralVerificationDelay)?;
        if delay.as_secs() > 0 {
            let verification_after = created_at + ChronoDuration::seconds(delay.as_secs() as i64);
            Ok(Some(verification_after.and_utc()))
        } else {
            Ok(None)
        }
    }

    fn add_new_referral(
        &mut self,
        referrer_username: &str,
        referred_username: &str,
        device_id: i32,
        risk_signal_id: i32,
        verified_at: Option<NaiveDateTime>,
    ) -> Result<(), DatabaseError> {
        RewardsStore::add_referral(
            self,
            NewRewardReferralRow {
                referrer_username: referrer_username.to_string(),
                referred_username: referred_username.to_string(),
                referred_device_id: device_id,
                risk_signal_id,
                verified_at,
            },
        )?;
        Ok(())
    }

    fn add_referral_verified_events(&mut self, referrer_username: &str, referred_username: &str) -> Result<Vec<RewardEvent>, DatabaseError> {
        let referrer_event = RewardsStore::add_event(
            self,
            NewRewardEventRow {
                username: referrer_username.to_string(),
                event_type: RewardEventType::InviteNew,
            },
            RewardEventType::InviteNew.points(),
        )?;

        let referred_event = RewardsStore::add_event(
            self,
            NewRewardEventRow {
                username: referred_username.to_string(),
                event_type: RewardEventType::Joined,
            },
            RewardEventType::Joined.points(),
        )?;

        Ok(vec![referrer_event.as_primitive(), referred_event.as_primitive()])
    }

    fn add_referral_pending_events(&mut self, referrer_username: &str) -> Result<Vec<RewardEvent>, DatabaseError> {
        let event = RewardsStore::add_event(
            self,
            NewRewardEventRow {
                username: referrer_username.to_string(),
                event_type: RewardEventType::InvitePending,
            },
            RewardEventType::InvitePending.points(),
        )?;
        Ok(vec![event.as_primitive()])
    }

    fn add_referral_with_events(
        &mut self,
        referrer_username: &str,
        referred_username: &str,
        device_id: i32,
        risk_signal_id: i32,
        verified_at: Option<NaiveDateTime>,
    ) -> Result<Vec<RewardEvent>, DatabaseError> {
        self.add_new_referral(referrer_username, referred_username, device_id, risk_signal_id, verified_at)?;

        if verified_at.is_some() {
            self.add_referral_verified_events(referrer_username, referred_username)
        } else {
            self.add_referral_pending_events(referrer_username)
        }
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
}
