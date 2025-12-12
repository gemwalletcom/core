use crate::DatabaseClient;
use crate::models::{NewReferral, NewReferralEvent, NewReferralUse, ReferralEvent, ReferralEventType, ReferralUse, StorageReferral};
use diesel::prelude::*;

pub enum ReferralLookup<'a> {
    Address(&'a str),
    Code(&'a str),
}

pub trait ReferralsEventTypesStore {
    fn add_referral_event_types(&mut self, event_types: Vec<ReferralEventType>) -> Result<usize, diesel::result::Error>;
}

impl ReferralsEventTypesStore for DatabaseClient {
    fn add_referral_event_types(&mut self, event_types: Vec<ReferralEventType>) -> Result<usize, diesel::result::Error> {
        use crate::schema::referrals_events_types::dsl;
        diesel::insert_into(dsl::referrals_events_types)
            .values(&event_types)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

pub(crate) trait ReferralsStore {
    fn get_referral(&mut self, lookup: ReferralLookup) -> Result<Option<StorageReferral>, diesel::result::Error>;
    fn create_referral(&mut self, referral: NewReferral) -> Result<StorageReferral, diesel::result::Error>;
    fn set_used_referral_code(&mut self, address: &str, referral_code: &str) -> Result<StorageReferral, diesel::result::Error>;
    fn add_referral_use(&mut self, referral_use: NewReferralUse) -> Result<(), diesel::result::Error>;
    fn get_referral_uses(&mut self, referrer_address: &str) -> Result<Vec<ReferralUse>, diesel::result::Error>;
    fn add_event(&mut self, event: NewReferralEvent) -> Result<(), diesel::result::Error>;
    fn get_events(&mut self, address: &str) -> Result<Vec<ReferralEvent>, diesel::result::Error>;
}

impl ReferralsStore for DatabaseClient {
    fn get_referral(&mut self, lookup: ReferralLookup) -> Result<Option<StorageReferral>, diesel::result::Error> {
        use crate::schema::referrals::dsl;
        match lookup {
            ReferralLookup::Address(address) => dsl::referrals
                .filter(dsl::address.eq(address))
                .select(StorageReferral::as_select())
                .first(&mut self.connection)
                .optional(),
            ReferralLookup::Code(code) => dsl::referrals
                .filter(dsl::code.eq(code))
                .select(StorageReferral::as_select())
                .first(&mut self.connection)
                .optional(),
        }
    }

    fn create_referral(&mut self, referral: NewReferral) -> Result<StorageReferral, diesel::result::Error> {
        use crate::schema::referrals::dsl;
        diesel::insert_into(dsl::referrals)
            .values(&referral)
            .returning(StorageReferral::as_returning())
            .get_result(&mut self.connection)
    }

    fn set_used_referral_code(&mut self, address: &str, referral_code: &str) -> Result<StorageReferral, diesel::result::Error> {
        use crate::schema::referrals::dsl;
        diesel::update(dsl::referrals.filter(dsl::address.eq(address)))
            .set(dsl::used_referral_code.eq(referral_code))
            .returning(StorageReferral::as_returning())
            .get_result(&mut self.connection)
    }

    fn add_referral_use(&mut self, referral_use: NewReferralUse) -> Result<(), diesel::result::Error> {
        use crate::schema::referrals_uses::dsl;
        diesel::insert_into(dsl::referrals_uses).values(&referral_use).execute(&mut self.connection)?;
        Ok(())
    }

    fn get_referral_uses(&mut self, address: &str) -> Result<Vec<ReferralUse>, diesel::result::Error> {
        use crate::schema::referrals_uses::dsl;
        dsl::referrals_uses
            .filter(dsl::referrer_address.eq(address))
            .order(dsl::created_at.desc())
            .select(ReferralUse::as_select())
            .load(&mut self.connection)
    }

    fn add_event(&mut self, event: NewReferralEvent) -> Result<(), diesel::result::Error> {
        use crate::schema::referrals_events::dsl;
        diesel::insert_into(dsl::referrals_events).values(&event).execute(&mut self.connection)?;
        Ok(())
    }

    fn get_events(&mut self, address: &str) -> Result<Vec<ReferralEvent>, diesel::result::Error> {
        use crate::schema::referrals_events::dsl;
        dsl::referrals_events
            .filter(dsl::address.eq(address))
            .order(dsl::created_at.desc())
            .select(ReferralEvent::as_select())
            .load(&mut self.connection)
    }
}
