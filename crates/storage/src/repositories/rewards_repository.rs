use crate::database::rewards::RewardsStore;
use crate::database::subscriptions::SubscriptionsStore;
use crate::database::usernames::{UsernameLookup, UsernamesStore};
use crate::models::{NewRewardEventRow, NewRewardReferralRow, ReferralAttemptRow, RewardsRow, UsernameRow};
use crate::repositories::rewards_redemptions_repository::RewardsRedemptionsRepository;
use crate::repositories::subscriptions_repository::SubscriptionsRepository;
use crate::{DatabaseClient, DatabaseError};
use chrono::NaiveDateTime;
use primitives::rewards::RewardRedemptionType;
use primitives::{Device, NaiveDateTimeExt, ReferralLeader, ReferralLeaderboard, RewardEvent, RewardEventType, Rewards, now};

fn has_custom_username(username: &str, address: &str) -> bool {
    !username.eq_ignore_ascii_case(address)
}

fn validate_username(username: &str) -> Result<(), DatabaseError> {
    let len = username.len();
    if len < 4 {
        return Err(DatabaseError::Internal("Username must be at least 4 characters".into()));
    }
    if len > 16 {
        return Err(DatabaseError::Internal("Username must be at most 16 characters".into()));
    }
    if !username.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(DatabaseError::Internal("Username must contain only letters and digits".into()));
    }
    Ok(())
}

pub trait RewardsRepository {
    fn get_reward_by_address(&mut self, address: &str) -> Result<Rewards, DatabaseError>;
    fn get_reward_events_by_address(&mut self, address: &str) -> Result<Vec<RewardEvent>, DatabaseError>;
    fn get_reward_event(&mut self, event_id: i32) -> Result<RewardEvent, DatabaseError>;
    fn get_reward_event_devices(&mut self, event_id: i32) -> Result<Vec<Device>, DatabaseError>;
    fn create_reward(&mut self, address: &str, username: &str) -> Result<(Rewards, i32), DatabaseError>;
    fn change_username(&mut self, address: &str, new_username: &str) -> Result<Rewards, DatabaseError>;
    fn referral_code_exists(&mut self, code: &str) -> Result<bool, DatabaseError>;
    fn validate_referral_use(&mut self, address: &str, referral_code: &str, device_id: i32) -> Result<(), DatabaseError>;
    fn create_referral_use(
        &mut self,
        address: &str,
        referral_code: &str,
        device_id: i32,
        ip_address: &str,
        invite_event: RewardEventType,
    ) -> Result<Vec<i32>, DatabaseError>;
    fn add_referral_attempt(
        &mut self,
        referrer_username: &str,
        referred_address: &str,
        device_id: i32,
        ip_address: &str,
        reason: &str,
    ) -> Result<(), DatabaseError>;
    fn get_first_subscription_date(&mut self, addresses: Vec<String>) -> Result<Option<NaiveDateTime>, DatabaseError>;
    fn get_address_by_username(&mut self, username: &str) -> Result<String, DatabaseError>;
    fn count_referrals_since(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn get_rewards_leaderboard(&mut self) -> Result<ReferralLeaderboard, DatabaseError>;
}

impl RewardsRepository for DatabaseClient {
    fn get_reward_by_address(&mut self, address: &str) -> Result<Rewards, DatabaseError> {
        let username = UsernamesStore::get_username(self, UsernameLookup::Address(address))?;
        let rewards = RewardsStore::get_rewards(self, &username.username)?;

        let code = if has_custom_username(&username.username, &username.address) {
            Some(username.username)
        } else {
            None
        };

        let options = if rewards.is_enabled {
            let types: Vec<_> = vec![RewardRedemptionType::Asset].iter().map(|t| t.as_ref().to_string()).collect();
            RewardsRedemptionsRepository::get_redemption_options(self, &types)?
        } else {
            vec![]
        };

        Ok(Rewards {
            code,
            referral_count: rewards.referral_count,
            points: rewards.points,
            used_referral_code: rewards.referrer_username,
            is_enabled: rewards.is_enabled,
            redemption_options: options,
        })
    }

    fn get_reward_events_by_address(&mut self, address: &str) -> Result<Vec<RewardEvent>, DatabaseError> {
        let username = UsernamesStore::get_username(self, UsernameLookup::Address(address))?;
        let events = RewardsStore::get_events(self, &username.username)?;
        Ok(events.iter().map(|e| e.as_primitive()).collect())
    }

    fn get_reward_event(&mut self, event_id: i32) -> Result<RewardEvent, DatabaseError> {
        let event = RewardsStore::get_event(self, event_id)?;
        Ok(event.as_primitive())
    }

    fn get_reward_event_devices(&mut self, event_id: i32) -> Result<Vec<Device>, DatabaseError> {
        let event = RewardsStore::get_event(self, event_id)?;
        let username = UsernamesStore::get_username(self, UsernameLookup::Username(&event.username))?;
        self.get_devices_by_address(&username.address)
    }

    fn create_reward(&mut self, address: &str, username: &str) -> Result<(Rewards, i32), DatabaseError> {
        validate_username(username)?;

        if UsernamesStore::username_exists(self, UsernameLookup::Username(username))? {
            return Err(DatabaseError::Internal("Username already taken".into()));
        }

        if UsernamesStore::username_exists(self, UsernameLookup::Address(address))? {
            let existing = UsernamesStore::get_username(self, UsernameLookup::Address(address))?;
            if has_custom_username(&existing.username, &existing.address) {
                return Err(DatabaseError::Internal("Address already has a username".into()));
            }
            let existing_rewards = RewardsStore::get_rewards(self, &existing.username)?;
            if !existing_rewards.is_enabled {
                return Err(DatabaseError::Internal("Rewards are not enabled for this user".into()));
            }
            UsernamesStore::update_username(self, address, username)?;
        } else {
            UsernamesStore::create_username(
                self,
                UsernameRow {
                    username: username.to_string(),
                    address: address.to_string(),
                    is_verified: false,
                },
            )?;
            RewardsStore::create_rewards(
                self,
                RewardsRow {
                    username: username.to_string(),
                    is_enabled: true,
                    level: None,
                    points: 0,
                    referrer_username: None,
                    referral_count: 0,
                },
            )?;
        }

        let event_id = RewardsStore::add_event(
            self,
            NewRewardEventRow {
                username: username.to_string(),
                event_type: RewardEventType::CreateUsername.as_ref().to_string(),
            },
            RewardEventType::CreateUsername.points(),
        )?;

        let rewards = self.get_reward_by_address(address)?;
        Ok((rewards, event_id))
    }

    fn change_username(&mut self, address: &str, new_username: &str) -> Result<Rewards, DatabaseError> {
        validate_username(new_username)?;

        let existing = UsernamesStore::get_username(self, UsernameLookup::Address(address))?;

        if !has_custom_username(&existing.username, &existing.address) {
            return Err(DatabaseError::Internal("No custom username to change".into()));
        }

        if existing.username.eq_ignore_ascii_case(new_username) {
            return Err(DatabaseError::Internal("New username is the same as current".into()));
        }

        if UsernamesStore::username_exists(self, UsernameLookup::Username(new_username))? {
            return Err(DatabaseError::Internal("Username already taken".into()));
        }

        let rewards = RewardsStore::get_rewards(self, &existing.username)?;
        if !rewards.is_enabled {
            return Err(DatabaseError::Internal("Rewards are not enabled for this user".into()));
        }

        UsernamesStore::change_username(self, &existing.username, new_username)?;

        self.get_reward_by_address(address)
    }

    fn referral_code_exists(&mut self, code: &str) -> Result<bool, DatabaseError> {
        Ok(RewardsStore::get_rewards(self, code).is_ok())
    }

    fn validate_referral_use(&mut self, address: &str, referral_code: &str, device_id: i32) -> Result<(), DatabaseError> {
        let referrer = UsernamesStore::get_username(self, UsernameLookup::Username(referral_code))?;
        let referrer_rewards = RewardsStore::get_rewards(self, &referrer.username)?;

        if !referrer_rewards.is_enabled {
            return Err(DatabaseError::Internal("Rewards are not enabled for this referral code".into()));
        }

        if UsernamesStore::username_exists(self, UsernameLookup::Address(address))? {
            let username = UsernamesStore::get_username(self, UsernameLookup::Address(address))?;
            let rewards = RewardsStore::get_rewards(self, &username.username)?;
            if !rewards.is_enabled {
                return Err(DatabaseError::Internal("Rewards are not enabled for this user".into()));
            }
            if rewards.referrer_username.is_some() {
                return Err(DatabaseError::Internal("Already used a referral code".into()));
            }
            if referrer.username == username.username || referrer.address.eq_ignore_ascii_case(&username.address) {
                return Err(DatabaseError::Internal("Cannot use your own referral code".into()));
            }
        }

        if SubscriptionsStore::get_device_subscription_address_exists(self, device_id, &referrer.address)? {
            return Err(DatabaseError::Internal("Cannot use your own referral code".into()));
        }

        if RewardsStore::get_referral_by_referred_device_id(self, device_id)?.is_some() {
            return Err(DatabaseError::Internal("Device already used a referral code".into()));
        }

        Ok(())
    }

    fn create_referral_use(
        &mut self,
        address: &str,
        referral_code: &str,
        device_id: i32,
        ip_address: &str,
        invite_event: RewardEventType,
    ) -> Result<Vec<i32>, DatabaseError> {
        let referrer = UsernamesStore::get_username(self, UsernameLookup::Username(referral_code))?;

        let referred = if UsernamesStore::username_exists(self, UsernameLookup::Address(address))? {
            UsernamesStore::get_username(self, UsernameLookup::Address(address))?
        } else {
            let username = UsernamesStore::create_username(
                self,
                UsernameRow {
                    username: address.to_string(),
                    address: address.to_string(),
                    is_verified: false,
                },
            )?;
            RewardsStore::create_rewards(
                self,
                RewardsRow {
                    username: address.to_string(),
                    is_enabled: true,
                    level: None,
                    points: 0,
                    referrer_username: None,
                    referral_count: 0,
                },
            )?;
            username
        };

        RewardsStore::add_referral(
            self,
            NewRewardReferralRow {
                referrer_username: referrer.username.clone(),
                referred_username: referred.username.clone(),
                referred_device_id: device_id,
                referred_ip_address: ip_address.to_string(),
            },
        )?;

        let invite_event_id = RewardsStore::add_event(
            self,
            NewRewardEventRow {
                username: referrer.username.clone(),
                event_type: invite_event.as_ref().to_string(),
            },
            invite_event.points(),
        )?;

        let joined_event_id = RewardsStore::add_event(
            self,
            NewRewardEventRow {
                username: referred.username.clone(),
                event_type: RewardEventType::Joined.as_ref().to_string(),
            },
            RewardEventType::Joined.points(),
        )?;

        Ok(vec![invite_event_id, joined_event_id])
    }

    fn add_referral_attempt(
        &mut self,
        referrer_username: &str,
        referred_address: &str,
        device_id: i32,
        ip_address: &str,
        reason: &str,
    ) -> Result<(), DatabaseError> {
        RewardsStore::add_referral_attempt(
            self,
            ReferralAttemptRow {
                referrer_username: referrer_username.to_string(),
                referred_address: referred_address.to_string(),
                device_id,
                referred_ip_address: ip_address.to_string(),
                reason: reason.to_string(),
            },
        )?;
        Ok(())
    }

    fn get_first_subscription_date(&mut self, addresses: Vec<String>) -> Result<Option<NaiveDateTime>, DatabaseError> {
        Ok(SubscriptionsStore::get_first_subscription_date(self, addresses)?)
    }

    fn get_address_by_username(&mut self, username: &str) -> Result<String, DatabaseError> {
        let username = UsernamesStore::get_username(self, UsernameLookup::Username(username))?;
        Ok(username.address)
    }

    fn count_referrals_since(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RewardsStore::count_referrals_since(self, referrer_username, since)?)
    }

    fn get_rewards_leaderboard(&mut self) -> Result<ReferralLeaderboard, DatabaseError> {
        let current = now();
        let limit = 10;

        let map_entry = |(username, referrals): (String, i64)| ReferralLeader {
            username,
            referrals: referrals as i32,
            points: referrals as i32 * RewardEventType::InviteNew.points(),
        };

        let daily = RewardsStore::get_top_referrers_since(self, current.days_ago(1), limit)?
            .into_iter()
            .map(map_entry)
            .collect();

        let weekly = RewardsStore::get_top_referrers_since(self, current.days_ago(7), limit)?
            .into_iter()
            .map(map_entry)
            .collect();

        let monthly = RewardsStore::get_top_referrers_since(self, current.days_ago(30), limit)?
            .into_iter()
            .map(map_entry)
            .collect();

        Ok(ReferralLeaderboard { daily, weekly, monthly })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_custom_username() {
        assert!(has_custom_username("alice", "0x1234567890abcdef"));
        assert!(!has_custom_username("0x1234567890abcdef", "0x1234567890abcdef"));
        assert!(!has_custom_username("0xABCDEF", "0xabcdef"));
        assert!(!has_custom_username("0xabcdef", "0xABCDEF"));
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
