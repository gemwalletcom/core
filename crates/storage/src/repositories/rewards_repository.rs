use crate::database::rewards::RewardsStore;
use crate::database::usernames::{UsernameLookup, UsernamesStore};
use crate::models::{NewRewardEvent, NewRewardReferral, Username};
use crate::{DatabaseClient, DatabaseError};
use primitives::{Rewards, RewardsEvent, RewardsEventItem};

fn has_custom_username(username: &str, address: &str) -> bool {
    !username.eq_ignore_ascii_case(address)
}

pub trait RewardsRepository {
    fn get_reward_by_address(&mut self, address: &str) -> Result<Rewards, DatabaseError>;
    fn get_reward_events_by_address(&mut self, address: &str) -> Result<Vec<RewardsEventItem>, DatabaseError>;
    fn get_reward_event(&mut self, event_id: i32) -> Result<(String, RewardsEvent), DatabaseError>;
    fn create_reward(&mut self, address: &str, username: &str) -> Result<(Rewards, i32), DatabaseError>;
    fn use_referral_code(&mut self, address: &str, referral_code: &str) -> Result<Vec<i32>, DatabaseError>;
}

impl RewardsRepository for DatabaseClient {
    fn get_reward_by_address(&mut self, address: &str) -> Result<Rewards, DatabaseError> {
        let user = UsernamesStore::get_username(self, UsernameLookup::Address(address))?;

        let referrals = RewardsStore::get_referrals_by_referrer(self, &user.username)?;
        let used_referral = RewardsStore::get_referral_by_referred(self, &user.username)?;
        let events = RewardsStore::get_events(self, &user.username)?;
        let total_points: i32 = events.iter().map(|e| e.as_primitive().points).sum();

        let code = if has_custom_username(&user.username, &user.address) {
            Some(user.username)
        } else {
            None
        };

        Ok(Rewards {
            code,
            referral_count: referrals.len() as i32,
            points: total_points,
            used_referral_code: used_referral.map(|r| r.referrer_username),
        })
    }

    fn get_reward_events_by_address(&mut self, address: &str) -> Result<Vec<RewardsEventItem>, DatabaseError> {
        let user = UsernamesStore::get_username(self, UsernameLookup::Address(address))?;
        let events = RewardsStore::get_events(self, &user.username)?;
        Ok(events.iter().map(|e| e.as_primitive()).collect())
    }

    fn get_reward_event(&mut self, event_id: i32) -> Result<(String, RewardsEvent), DatabaseError> {
        let event = RewardsStore::get_event(self, event_id)?;
        let user = UsernamesStore::get_username(self, UsernameLookup::Username(&event.username))?;
        Ok((user.address, event.as_primitive().event))
    }

    fn create_reward(&mut self, address: &str, username: &str) -> Result<(Rewards, i32), DatabaseError> {
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
                Username {
                    username: username.to_string(),
                    address: address.to_string(),
                },
            )?;
        }

        let event_id = RewardsStore::add_event(
            self,
            NewRewardEvent {
                username: username.to_string(),
                event_type: RewardsEvent::CreateUsername.as_ref().to_string(),
            },
        )?;
        let rewards = self.get_reward_by_address(address)?;
        Ok((rewards, event_id))
    }

    fn use_referral_code(&mut self, address: &str, referral_code: &str) -> Result<Vec<i32>, DatabaseError> {
        if !UsernamesStore::username_exists(self, UsernameLookup::Username(referral_code))? {
            return Err(DatabaseError::Internal("Referral code does not exist".into()));
        }
        let referrer = UsernamesStore::get_username(self, UsernameLookup::Username(referral_code))?;

        let user = if UsernamesStore::username_exists(self, UsernameLookup::Address(address))? {
            UsernamesStore::get_username(self, UsernameLookup::Address(address))?
        } else {
            UsernamesStore::create_username(
                self,
                Username {
                    username: address.to_string(),
                    address: address.to_string(),
                },
            )?
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
        let invite_event_id = RewardsStore::add_event(
            self,
            NewRewardEvent {
                username: referrer.username,
                event_type: RewardsEvent::Invite.as_ref().to_string(),
            },
        )?;
        let joined_event_id = RewardsStore::add_event(
            self,
            NewRewardEvent {
                username: user.username,
                event_type: RewardsEvent::Joined.as_ref().to_string(),
            },
        )?;

        Ok(vec![invite_event_id, joined_event_id])
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
