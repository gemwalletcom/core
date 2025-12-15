use crate::database::rewards::RewardsStore;
use crate::database::subscriptions::SubscriptionsStore;
use crate::database::usernames::{UsernameLookup, UsernamesStore};
use crate::models::{NewRewardEventRow, NewRewardReferralRow, UsernameRow};
use crate::repositories::subscriptions_repository::SubscriptionsRepository;
use crate::{DatabaseClient, DatabaseError};
use chrono::NaiveDateTime;
use primitives::{Device, RewardEvent, RewardEventType, RewardLevel, Rewards};
use std::str::FromStr;

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
    fn use_referral_code(&mut self, address: &str, referral_code: &str, device_id: i32, invite_event: RewardEventType) -> Result<Vec<i32>, DatabaseError>;
    fn get_first_subscription_date(&mut self, addresses: Vec<String>) -> Result<Option<NaiveDateTime>, DatabaseError>;
}

impl RewardsRepository for DatabaseClient {
    fn get_reward_by_address(&mut self, address: &str) -> Result<Rewards, DatabaseError> {
        let username = UsernamesStore::get_username(self, UsernameLookup::Address(address))?;

        let referrals = RewardsStore::get_referrals_by_referrer(self, &username.username)?;
        let used_referral = RewardsStore::get_referral_by_referred(self, &username.username)?;
        let events = RewardsStore::get_events(self, &username.username)?;
        let total_points: i32 = events.iter().map(|e| e.as_primitive().points).sum();

        let code = if has_custom_username(&username.username, &username.address) {
            Some(username.username)
        } else {
            None
        };

        let level = username.rewards_level.as_ref().and_then(|l| RewardLevel::from_str(l).ok());

        Ok(Rewards {
            code,
            referral_count: referrals.len() as i32,
            points: total_points,
            used_referral_code: used_referral.map(|r| r.referrer_username),
            is_enabled: username.is_rewards_enabled,
            level,
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
            UsernamesStore::update_username(self, address, username)?;
        } else {
            UsernamesStore::create_username(
                self,
                UsernameRow {
                    username: username.to_string(),
                    address: address.to_string(),
                    is_verified: false,
                    is_rewards_enabled: true,
                    rewards_level: None,
                },
            )?;
        }

        let event_id = RewardsStore::add_event(
            self,
            NewRewardEventRow {
                username: username.to_string(),
                event_type: RewardEventType::CreateUsername.as_ref().to_string(),
            },
        )?;
        let rewards = self.get_reward_by_address(address)?;
        Ok((rewards, event_id))
    }

    fn use_referral_code(&mut self, address: &str, referral_code: &str, device_id: i32, invite_event: RewardEventType) -> Result<Vec<i32>, DatabaseError> {
        if !UsernamesStore::username_exists(self, UsernameLookup::Username(referral_code))? {
            return Err(DatabaseError::Internal("Referral code does not exist".into()));
        }
        let referrer = UsernamesStore::get_username(self, UsernameLookup::Username(referral_code))?;

        if !referrer.is_rewards_enabled {
            return Err(DatabaseError::Internal("Rewards are not enabled for this referral code".into()));
        }

        let referred = if UsernamesStore::username_exists(self, UsernameLookup::Address(address))? {
            UsernamesStore::get_username(self, UsernameLookup::Address(address))?
        } else {
            UsernamesStore::create_username(
                self,
                UsernameRow {
                    username: address.to_string(),
                    address: address.to_string(),
                    is_verified: false,
                    is_rewards_enabled: true,
                    rewards_level: None,
                },
            )?
        };

        if referrer.username == referred.username || referrer.address.eq_ignore_ascii_case(&referred.address) {
            return Err(DatabaseError::Internal("Cannot use your own referral code".into()));
        }

        if SubscriptionsStore::get_device_subscription_address_exists(self, device_id, &referrer.address)? {
            return Err(DatabaseError::Internal("Cannot use your own referral code".into()));
        }

        if RewardsStore::get_referral_by_referred(self, &referred.username)?.is_some() {
            return Err(DatabaseError::Internal("Already used a referral code".into()));
        }

        if RewardsStore::get_referral_by_referred_device_id(self, device_id)?.is_some() {
            return Err(DatabaseError::Internal("Device already used a referral code".into()));
        }

        if !referred.is_rewards_enabled {
            return Err(DatabaseError::Internal("Rewards are not enabled for this user".into()));
        }

        RewardsStore::add_referral(
            self,
            NewRewardReferralRow {
                referrer_username: referral_code.to_string(),
                referred_username: referred.username.clone(),
                referred_device_id: device_id,
            },
        )?;
        let invite_event_id = RewardsStore::add_event(
            self,
            NewRewardEventRow {
                username: referrer.username,
                event_type: invite_event.as_ref().to_string(),
            },
        )?;
        let joined_event_id = RewardsStore::add_event(
            self,
            NewRewardEventRow {
                username: referred.username,
                event_type: RewardEventType::Joined.as_ref().to_string(),
            },
        )?;

        Ok(vec![invite_event_id, joined_event_id])
    }

    fn get_first_subscription_date(&mut self, addresses: Vec<String>) -> Result<Option<NaiveDateTime>, DatabaseError> {
        Ok(SubscriptionsStore::get_first_subscription_date(self, addresses)?)
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
