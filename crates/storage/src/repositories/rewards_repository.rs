use crate::database::rewards::RewardsStore;
use crate::database::usernames::{UsernameLookup, UsernamesStore};
use crate::models::{NewRewardEvent, NewRewardReferral, Username};
use crate::{DatabaseClient, DatabaseError};
use chrono::{TimeZone, Utc};
use primitives::RewardsEvent;
use std::str::FromStr;

fn has_custom_username(username: &str, address: &str) -> bool {
    !username.eq_ignore_ascii_case(address)
}

pub trait RewardsRepository {
    fn get_reward_by_address(&mut self, address: &str) -> Result<primitives::Rewards, DatabaseError>;
    fn get_reward_events_by_address(&mut self, address: &str) -> Result<Vec<primitives::RewardsEventItem>, DatabaseError>;
    fn create_reward(&mut self, address: &str, username: &str) -> Result<primitives::Rewards, DatabaseError>;
    fn use_referral_code(&mut self, address: &str, referral_code: &str) -> Result<(), DatabaseError>;
}

impl RewardsRepository for DatabaseClient {
    fn get_reward_by_address(&mut self, address: &str) -> Result<primitives::Rewards, DatabaseError> {
        let user = match UsernamesStore::get_username(self, UsernameLookup::Address(address))? {
            Some(u) => u,
            None => {
                return Ok(primitives::Rewards {
                    code: None,
                    referral_count: 0,
                    points: 0,
                    used_referral_code: None,
                });
            }
        };

        let referrals = RewardsStore::get_referrals_by_referrer(self, &user.username)?;
        let used_referral = RewardsStore::get_referral_by_referred(self, &user.username)?;
        let events = RewardsStore::get_events(self, &user.username)?;
        let total_points: i32 = events
            .iter()
            .filter_map(|e| primitives::RewardsEvent::from_str(&e.event_type).ok())
            .map(|e| e.points())
            .sum();

        let code = if has_custom_username(&user.username, &user.address) {
            Some(user.username)
        } else {
            None
        };

        Ok(primitives::Rewards {
            code,
            referral_count: referrals.len() as i32,
            points: total_points,
            used_referral_code: used_referral.map(|r| r.referrer_username),
        })
    }

    fn get_reward_events_by_address(&mut self, address: &str) -> Result<Vec<primitives::RewardsEventItem>, DatabaseError> {
        let user = match UsernamesStore::get_username(self, UsernameLookup::Address(address))? {
            Some(u) => u,
            None => return Ok(vec![]),
        };

        let events = RewardsStore::get_events(self, &user.username)?;
        Ok(events
            .into_iter()
            .filter_map(|e| {
                primitives::RewardsEvent::from_str(&e.event_type)
                    .ok()
                    .map(|event| primitives::RewardsEventItem {
                        points: event.points(),
                        event,
                        created_at: Utc.from_utc_datetime(&e.created_at),
                    })
            })
            .collect())
    }

    fn create_reward(&mut self, address: &str, username: &str) -> Result<primitives::Rewards, DatabaseError> {
        if UsernamesStore::get_username(self, UsernameLookup::Username(username))?.is_some() {
            return Err(DatabaseError::Internal("Username already taken".into()));
        }

        if let Some(existing) = UsernamesStore::get_username(self, UsernameLookup::Address(address))? {
            if has_custom_username(&existing.username, &existing.address) {
                return Err(DatabaseError::Internal("Address already has a username".into()));
            }
            UsernamesStore::update_username(self, address, username)?;
        } else {
            UsernamesStore::create_username(
                self,
                Username {
                    username: username.to_string(),
                    address: address.to_string(),
                },
            )?;
        }

        RewardsStore::add_event(
            self,
            NewRewardEvent {
                username: username.to_string(),
                event_type: RewardsEvent::CreateUsername.as_ref().to_string(),
            },
        )?;
        self.get_reward_by_address(address)
    }

    fn use_referral_code(&mut self, address: &str, referral_code: &str) -> Result<(), DatabaseError> {
        let referrer = UsernamesStore::get_username(self, UsernameLookup::Username(referral_code))?
            .ok_or_else(|| DatabaseError::Internal("Referral code does not exist".into()))?;

        let user = match UsernamesStore::get_username(self, UsernameLookup::Address(address))? {
            Some(u) => u,
            None => {
                UsernamesStore::create_username(
                    self,
                    Username {
                        username: address.to_string(),
                        address: address.to_string(),
                    },
                )?
            }
        };

        if referrer.username == user.username || referrer.address.eq_ignore_ascii_case(&user.address) {
            return Err(DatabaseError::Internal("Cannot use your own referral code".into()));
        }

        if RewardsStore::get_referral_by_referred(self, &user.username)?.is_some() {
            return Err(DatabaseError::Internal("Already used a referral code".into()));
        }

        RewardsStore::add_referral(
            self,
            NewRewardReferral {
                referrer_username: referral_code.to_string(),
                referred_username: user.username.clone(),
            },
        )?;
        RewardsStore::add_event(
            self,
            NewRewardEvent {
                username: referrer.username,
                event_type: RewardsEvent::Invite.as_ref().to_string(),
            },
        )?;
        RewardsStore::add_event(
            self,
            NewRewardEvent {
                username: user.username,
                event_type: RewardsEvent::Joined.as_ref().to_string(),
            },
        )?;

        Ok(())
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
}
