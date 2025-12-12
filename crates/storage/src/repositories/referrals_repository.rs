use crate::database::referrals::{ReferralLookup, ReferralsStore};
use crate::models::{NewReferral, NewReferralEvent, NewReferralUse};
use crate::{DatabaseClient, DatabaseError};
use chrono::{TimeZone, Utc};
use std::str::FromStr;

pub trait ReferralsRepository {
    fn get_referral(&mut self, address: &str) -> Result<primitives::Referral, DatabaseError>;
    fn create_referral(&mut self, address: &str, code: &str) -> Result<primitives::Referral, DatabaseError>;
    fn use_referral_code(&mut self, address: &str, referral_code: &str) -> Result<(), DatabaseError>;
    fn get_referral_events(&mut self, address: &str) -> Result<Vec<primitives::ReferralEventItem>, DatabaseError>;
}

impl ReferralsRepository for DatabaseClient {
    fn get_referral(&mut self, address: &str) -> Result<primitives::Referral, DatabaseError> {
        let referral = ReferralsStore::get_referral(self, ReferralLookup::Address(address))?.ok_or(DatabaseError::NotFound)?;
        let uses = ReferralsStore::get_referral_uses(self, &referral.address)?;
        let events = self.get_referral_events(&referral.address)?;
        let total_points = events.iter().map(|e| e.points).sum();
        Ok(referral.as_primitive(uses.len() as i32, total_points))
    }

    fn create_referral(&mut self, address: &str, code: &str) -> Result<primitives::Referral, DatabaseError> {
        if ReferralsStore::get_referral(self, ReferralLookup::Address(address))?.is_some() {
            return Err(DatabaseError::Internal("Referral already exists".into()));
        }

        if ReferralsStore::get_referral(self, ReferralLookup::Code(code))?.is_some() {
            return Err(DatabaseError::Internal("Referral code already taken".into()));
        }

        let new_referral = NewReferral {
            address: address.to_string(),
            code: Some(code.to_string()),
        };
        let referral = ReferralsStore::create_referral(self, new_referral)?;
        Ok(referral.as_primitive(0, 0))
    }

    fn use_referral_code(&mut self, address: &str, referral_code: &str) -> Result<(), DatabaseError> {
        let referrer = ReferralsStore::get_referral(self, ReferralLookup::Code(referral_code))?
            .ok_or_else(|| DatabaseError::Internal("Referral code does not exist".into()))?;

        if referrer.address == address {
            return Err(DatabaseError::Internal("Cannot use your own referral code".into()));
        }

        if let Some(existing) = ReferralsStore::get_referral(self, ReferralLookup::Address(address))? {
            if existing.used_referral_code.is_some() {
                return Err(DatabaseError::Internal("Already used a referral code".into()));
            }
            if existing.code.is_some() {
                return Err(DatabaseError::Internal("Cannot use referral code after creating your own".into()));
            }
        } else {
            ReferralsStore::create_referral(self, NewReferral { address: address.to_string(), code: None })?;
        }

        ReferralsStore::set_used_referral_code(self, address, referral_code)?;
        ReferralsStore::add_referral_use(self, NewReferralUse {
            referrer_address: referrer.address.clone(),
            referred_address: address.to_string(),
        })?;
        ReferralsStore::add_event(self, NewReferralEvent {
            address: referrer.address,
            event_type: primitives::ReferralEvent::Invite.as_ref().to_string(),
        })?;

        Ok(())
    }

    fn get_referral_events(&mut self, address: &str) -> Result<Vec<primitives::ReferralEventItem>, DatabaseError> {
        let events = ReferralsStore::get_events(self, address)?;
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
}
