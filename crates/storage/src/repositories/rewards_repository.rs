use crate::database::rewards::RewardsStore;
use crate::database::usernames::{UsernameLookup, UsernamesStore};
use crate::models::{NewRewardEvent, NewRewardReferral, Username};
use crate::{DatabaseClient, DatabaseError};
use chrono::{TimeZone, Utc};
use primitives::ReferralEvent;
use std::str::FromStr;

pub trait RewardsRepository {
    fn get_reward_by_address(&mut self, address: &str) -> Result<primitives::Referral, DatabaseError>;
    fn get_reward_events_by_address(&mut self, address: &str) -> Result<Vec<primitives::ReferralEventItem>, DatabaseError>;
    fn create_reward(&mut self, address: &str, username: &str) -> Result<primitives::Referral, DatabaseError>;
    fn use_referral_code(&mut self, address: &str, referral_code: &str) -> Result<(), DatabaseError>;
}

impl RewardsRepository for DatabaseClient {
    fn get_reward_by_address(&mut self, address: &str) -> Result<primitives::Referral, DatabaseError> {
        let user = UsernamesStore::get_username(self, UsernameLookup::Address(address))?.ok_or(DatabaseError::NotFound)?;
        let referrals = RewardsStore::get_referrals_by_referrer(self, &user.username)?;
        let used_referral = RewardsStore::get_referral_by_referred(self, &user.username)?;
        let events = self.get_reward_events_by_address(address)?;
        let total_points = events.iter().map(|e| e.points).sum();
        Ok(primitives::Referral {
            code: Some(user.username),
            referral_count: referrals.len() as i32,
            points: total_points,
            used_referral_code: used_referral.map(|r| r.referrer_username),
        })
    }

    fn get_reward_events_by_address(&mut self, address: &str) -> Result<Vec<primitives::ReferralEventItem>, DatabaseError> {
        let user = UsernamesStore::get_username(self, UsernameLookup::Address(address))?.ok_or(DatabaseError::NotFound)?;
        let events = RewardsStore::get_events(self, &user.username)?;
        Ok(events
            .into_iter()
            .filter_map(|e| {
                primitives::ReferralEvent::from_str(&e.event_type)
                    .ok()
                    .map(|event| primitives::ReferralEventItem {
                        points: event.points(),
                        event,
                        created_at: Utc.from_utc_datetime(&e.created_at),
                    })
            })
            .collect())
    }

    fn create_reward(&mut self, address: &str, username: &str) -> Result<primitives::Referral, DatabaseError> {
        if UsernamesStore::get_username(self, UsernameLookup::Username(username))?.is_some() {
            return Err(DatabaseError::Internal("Username already taken".into()));
        }
        if UsernamesStore::get_username(self, UsernameLookup::Address(address))?.is_some() {
            return Err(DatabaseError::Internal("Address already has a username".into()));
        }

        UsernamesStore::create_username(
            self,
            Username {
                username: username.to_string(),
                address: address.to_string(),
            },
        )?;
        RewardsStore::add_event(
            self,
            NewRewardEvent {
                username: username.to_string(),
                event_type: ReferralEvent::CreateUsername.as_ref().to_string(),
            },
        )?;
        self.get_reward_by_address(address)
    }

    fn use_referral_code(&mut self, address: &str, referral_code: &str) -> Result<(), DatabaseError> {
        let user = UsernamesStore::get_username(self, UsernameLookup::Address(address))?.ok_or(DatabaseError::NotFound)?;

        let referrer = UsernamesStore::get_username(self, UsernameLookup::Username(referral_code))?
            .ok_or_else(|| DatabaseError::Internal("Referral code does not exist".into()))?;

        if referrer.username == user.username {
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
                event_type: ReferralEvent::Invite.as_ref().to_string(),
            },
        )?;
        RewardsStore::add_event(
            self,
            NewRewardEvent {
                username: user.username,
                event_type: ReferralEvent::Joined.as_ref().to_string(),
            },
        )?;

        Ok(())
    }
}
