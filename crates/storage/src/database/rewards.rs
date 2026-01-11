use crate::DatabaseClient;
use crate::models::{
    NewRewardEventRow, NewRewardReferralRow, NewRewardsRow, NewRiskSignalRow, ReferralAttemptRow, RewardEventRow, RewardEventTypeRow, RewardReferralRow,
    RewardsRow, RiskSignalRow,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum ReferralUpdate {
    VerifiedAt(NaiveDateTime),
}

#[derive(Debug, Clone, Default)]
pub struct AbusePatterns {
    pub max_countries_per_device: i64,
    pub max_referrers_per_device: i64,
    pub max_referrers_per_fingerprint: i64,
    pub max_devices_per_ip: i64,
}

pub trait RewardsEventTypesStore {
    fn add_reward_event_types(&mut self, event_types: Vec<RewardEventTypeRow>) -> Result<usize, DieselError>;
}

impl RewardsEventTypesStore for DatabaseClient {
    fn add_reward_event_types(&mut self, event_types: Vec<RewardEventTypeRow>) -> Result<usize, DieselError> {
        use crate::schema::rewards_events_types::dsl;
        diesel::insert_into(dsl::rewards_events_types)
            .values(&event_types)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

pub(crate) trait RewardsStore {
    fn get_rewards(&mut self, username: &str) -> Result<RewardsRow, DieselError>;
    fn create_rewards(&mut self, rewards: NewRewardsRow) -> Result<RewardsRow, DieselError>;
    fn add_referral(&mut self, referral: NewRewardReferralRow) -> Result<(), DieselError>;
    fn get_referral_by_referred_device_id(&mut self, referred_device_id: i32) -> Result<Option<RewardReferralRow>, DieselError>;
    fn get_referral_by_username(&mut self, username: &str) -> Result<Option<RewardReferralRow>, DieselError>;
    fn update_referral(&mut self, referral_id: i32, update: ReferralUpdate) -> Result<(), DieselError>;
    fn add_referral_attempt(&mut self, attempt: ReferralAttemptRow) -> Result<(), DieselError>;
    fn add_event(&mut self, event: NewRewardEventRow, points: i32) -> Result<RewardEventRow, DieselError>;
    fn get_event(&mut self, event_id: i32) -> Result<RewardEventRow, DieselError>;
    fn get_events(&mut self, username: &str) -> Result<Vec<RewardEventRow>, DieselError>;
    fn count_referrals_since(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DieselError>;
    fn get_top_referrers_since(&mut self, event_types: Vec<String>, since: NaiveDateTime, limit: i64) -> Result<Vec<(String, i64, i64)>, DieselError>;
    fn disable_rewards(&mut self, username: &str, reason: &str, comment: &str) -> Result<i32, DieselError>;
}

impl RewardsStore for DatabaseClient {
    fn get_rewards(&mut self, username: &str) -> Result<RewardsRow, DieselError> {
        use crate::schema::rewards::dsl;
        dsl::rewards
            .filter(dsl::username.eq(username))
            .select(RewardsRow::as_select())
            .first(&mut self.connection)
    }

    fn create_rewards(&mut self, rewards: NewRewardsRow) -> Result<RewardsRow, DieselError> {
        use crate::schema::rewards::dsl;
        diesel::insert_into(dsl::rewards)
            .values(&rewards)
            .returning(RewardsRow::as_returning())
            .get_result(&mut self.connection)
    }

    fn add_referral(&mut self, referral: NewRewardReferralRow) -> Result<(), DieselError> {
        use crate::schema::{rewards, rewards_referrals};
        use diesel::Connection;

        self.connection.transaction(|conn| {
            diesel::insert_into(rewards_referrals::table).values(&referral).execute(conn)?;

            diesel::update(rewards::table.filter(rewards::username.eq(&referral.referred_username)))
                .set(rewards::referrer_username.eq(&referral.referrer_username))
                .execute(conn)?;

            diesel::update(rewards::table.filter(rewards::username.eq(&referral.referrer_username)))
                .set(rewards::referral_count.eq(rewards::referral_count + 1))
                .execute(conn)?;

            Ok(())
        })
    }

    fn get_referral_by_referred_device_id(&mut self, referred_device_id: i32) -> Result<Option<RewardReferralRow>, DieselError> {
        use crate::schema::rewards_referrals::dsl;
        dsl::rewards_referrals
            .filter(dsl::referred_device_id.eq(referred_device_id))
            .select(RewardReferralRow::as_select())
            .first(&mut self.connection)
            .optional()
    }

    fn add_referral_attempt(&mut self, attempt: ReferralAttemptRow) -> Result<(), DieselError> {
        use crate::schema::rewards_referral_attempts::dsl;
        diesel::insert_into(dsl::rewards_referral_attempts)
            .values(&attempt)
            .execute(&mut self.connection)?;
        Ok(())
    }

    fn add_event(&mut self, new_event: NewRewardEventRow, points: i32) -> Result<RewardEventRow, DieselError> {
        use crate::schema::{rewards, rewards_events};
        use diesel::Connection;

        if points < 0 {
            return Err(DieselError::RollbackTransaction);
        }

        self.connection.transaction(|conn| {
            let event = diesel::insert_into(rewards_events::table)
                .values(&new_event)
                .returning(RewardEventRow::as_returning())
                .get_result(conn)?;

            diesel::update(rewards::table.filter(rewards::username.eq(&new_event.username)))
                .set(rewards::points.eq(rewards::points + points))
                .returning(rewards::username)
                .get_result::<String>(conn)?;

            Ok(event)
        })
    }

    fn get_event(&mut self, event_id: i32) -> Result<RewardEventRow, DieselError> {
        use crate::schema::rewards_events::dsl;
        dsl::rewards_events
            .filter(dsl::id.eq(event_id))
            .select(RewardEventRow::as_select())
            .first(&mut self.connection)
    }

    fn get_events(&mut self, username: &str) -> Result<Vec<RewardEventRow>, DieselError> {
        use crate::schema::rewards_events::dsl;
        dsl::rewards_events
            .filter(dsl::username.eq(username))
            .order(dsl::created_at.desc())
            .select(RewardEventRow::as_select())
            .load(&mut self.connection)
    }

    fn count_referrals_since(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_referrals::dsl;
        dsl::rewards_referrals
            .filter(dsl::referrer_username.eq(referrer_username))
            .filter(dsl::created_at.ge(since))
            .count()
            .get_result(&mut self.connection)
    }

    fn get_top_referrers_since(&mut self, event_types: Vec<String>, since: NaiveDateTime, limit: i64) -> Result<Vec<(String, i64, i64)>, DieselError> {
        use crate::schema::{rewards, rewards_events, rewards_events_types};
        use diesel::dsl::{count_star, sum};

        rewards_events::table
            .inner_join(rewards::table.on(rewards_events::username.eq(rewards::username)))
            .inner_join(rewards_events_types::table.on(rewards_events::event_type.eq(rewards_events_types::id)))
            .filter(rewards::is_enabled.eq(true))
            .filter(rewards_events::event_type.eq_any(event_types))
            .filter(rewards_events::created_at.ge(since))
            .group_by(rewards_events::username)
            .select((rewards_events::username, count_star(), sum(rewards_events_types::points).assume_not_null()))
            .order_by(sum(rewards_events_types::points).desc())
            .limit(limit)
            .load(&mut self.connection)
    }

    fn disable_rewards(&mut self, username: &str, reason: &str, comment: &str) -> Result<i32, DieselError> {
        use crate::schema::{rewards, rewards_events};
        use diesel::Connection;
        use primitives::RewardEventType;

        self.connection.transaction(|conn| {
            diesel::update(rewards::table.filter(rewards::username.eq(username)))
                .set((rewards::is_enabled.eq(false), rewards::disable_reason.eq(reason), rewards::comment.eq(comment)))
                .execute(conn)?;

            let event_id = diesel::insert_into(rewards_events::table)
                .values(NewRewardEventRow {
                    username: username.to_string(),
                    event_type: RewardEventType::Disabled.as_ref().to_string(),
                })
                .returning(rewards_events::id)
                .get_result(conn)?;

            Ok(event_id)
        })
    }

    fn get_referral_by_username(&mut self, username: &str) -> Result<Option<RewardReferralRow>, DieselError> {
        use crate::schema::rewards_referrals::dsl;
        dsl::rewards_referrals
            .filter(dsl::referred_username.eq(username))
            .first(&mut self.connection)
            .optional()
    }

    fn update_referral(&mut self, referral_id: i32, update: ReferralUpdate) -> Result<(), DieselError> {
        use crate::schema::rewards_referrals::dsl;
        match update {
            ReferralUpdate::VerifiedAt(timestamp) => {
                diesel::update(dsl::rewards_referrals.find(referral_id))
                    .set(dsl::verified_at.eq(timestamp))
                    .execute(&mut self.connection)?;
            }
        }
        Ok(())
    }
}

pub(crate) trait RiskSignalsStore {
    fn add_risk_signal(&mut self, signal: NewRiskSignalRow) -> Result<i32, DieselError>;
    fn has_fingerprint_for_referrer(&mut self, fingerprint: &str, referrer_username: &str, since: NaiveDateTime) -> Result<bool, DieselError>;
    fn get_matching_risk_signals(
        &mut self,
        fingerprint: &str,
        ip_address: &str,
        ip_isp: &str,
        device_model: &str,
        device_id: i32,
        since: NaiveDateTime,
    ) -> Result<Vec<RiskSignalRow>, DieselError>;
    fn count_signals_since(&mut self, ip_address: Option<&str>, since: NaiveDateTime) -> Result<i64, DieselError>;
    fn count_signals_for_device_id(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DieselError>;
    fn count_signals_for_country(&mut self, country_code: &str, since: NaiveDateTime) -> Result<i64, DieselError>;
    fn sum_risk_scores_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DieselError>;
    fn count_attempts_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DieselError>;
    fn get_referrer_usernames_with_referrals(&mut self, since: NaiveDateTime, min_referrals: i64) -> Result<Vec<String>, DieselError>;
    fn count_unique_countries_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DieselError>;
    fn count_unique_referrers_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DieselError>;
    fn count_unique_referrers_for_fingerprint(&mut self, fingerprint: &str, since: NaiveDateTime) -> Result<i64, DieselError>;
    fn count_unique_devices_for_ip(&mut self, ip_address: &str, since: NaiveDateTime) -> Result<i64, DieselError>;
    fn count_unique_referrers_for_device_model_pattern(
        &mut self,
        device_model: &str,
        device_platform: &str,
        device_locale: &str,
        since: NaiveDateTime,
    ) -> Result<i64, DieselError>;
    fn get_abuse_patterns_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<AbusePatterns, DieselError>;
}

impl RiskSignalsStore for DatabaseClient {
    fn add_risk_signal(&mut self, signal: NewRiskSignalRow) -> Result<i32, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;
        diesel::insert_into(dsl::rewards_risk_signals)
            .values(&signal)
            .returning(dsl::id)
            .get_result(&mut self.connection)
    }

    fn has_fingerprint_for_referrer(&mut self, fingerprint: &str, referrer_username: &str, since: NaiveDateTime) -> Result<bool, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::exists;

        diesel::select(exists(
            dsl::rewards_risk_signals
                .filter(dsl::fingerprint.eq(fingerprint))
                .filter(dsl::referrer_username.eq(referrer_username))
                .filter(dsl::created_at.ge(since)),
        ))
        .get_result(&mut self.connection)
    }

    fn get_matching_risk_signals(
        &mut self,
        fingerprint: &str,
        ip_address: &str,
        ip_isp: &str,
        device_model: &str,
        device_id: i32,
        since: NaiveDateTime,
    ) -> Result<Vec<RiskSignalRow>, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;

        dsl::rewards_risk_signals
            .filter(dsl::created_at.ge(since))
            .filter(
                dsl::fingerprint
                    .eq(fingerprint)
                    .or(dsl::ip_address.eq(ip_address))
                    .or(dsl::ip_isp.eq(ip_isp).and(dsl::device_model.eq(device_model)))
                    .or(dsl::device_id.eq(device_id)),
            )
            .order(dsl::created_at.desc())
            .limit(100)
            .select(RiskSignalRow::as_select())
            .load(&mut self.connection)
    }

    fn count_signals_since(&mut self, ip_address: Option<&str>, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_referrals;
        use crate::schema::rewards_risk_signals::dsl;

        let mut query = dsl::rewards_risk_signals
            .inner_join(rewards_referrals::table.on(rewards_referrals::risk_signal_id.eq(dsl::id)))
            .filter(dsl::created_at.ge(since))
            .into_boxed();

        if let Some(ip) = ip_address {
            query = query.filter(dsl::ip_address.eq(ip));
        }

        query.count().get_result(&mut self.connection)
    }

    fn count_signals_for_device_id(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;

        dsl::rewards_risk_signals
            .filter(dsl::device_id.eq(device_id))
            .filter(dsl::created_at.ge(since))
            .count()
            .get_result(&mut self.connection)
    }

    fn count_signals_for_country(&mut self, country_code: &str, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;

        dsl::rewards_risk_signals
            .filter(dsl::ip_country_code.eq(country_code))
            .filter(dsl::created_at.ge(since))
            .count()
            .get_result(&mut self.connection)
    }

    fn sum_risk_scores_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::sum;

        dsl::rewards_risk_signals
            .filter(dsl::referrer_username.eq(referrer_username))
            .filter(dsl::created_at.ge(since))
            .select(sum(dsl::risk_score))
            .first::<Option<i64>>(&mut self.connection)
            .map(|s| s.unwrap_or(0))
    }

    fn count_attempts_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_referral_attempts::dsl;

        dsl::rewards_referral_attempts
            .filter(dsl::referrer_username.eq(referrer_username))
            .filter(dsl::created_at.ge(since))
            .filter(dsl::risk_signal_id.is_not_null())
            .count()
            .get_result(&mut self.connection)
    }

    fn get_referrer_usernames_with_referrals(&mut self, since: NaiveDateTime, min_referrals: i64) -> Result<Vec<String>, DieselError> {
        use crate::schema::{rewards, rewards_referrals};
        use diesel::dsl::count_star;

        rewards_referrals::table
            .inner_join(rewards::table.on(rewards_referrals::referrer_username.eq(rewards::username)))
            .filter(rewards::is_enabled.eq(true))
            .filter(rewards_referrals::created_at.ge(since))
            .group_by(rewards_referrals::referrer_username)
            .having(count_star().ge(min_referrals))
            .select(rewards_referrals::referrer_username)
            .load(&mut self.connection)
    }

    fn count_unique_countries_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        dsl::rewards_risk_signals
            .filter(dsl::device_id.eq(device_id))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::ip_country_code).aggregate_distinct())
            .first(&mut self.connection)
    }

    fn count_unique_referrers_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        dsl::rewards_risk_signals
            .filter(dsl::device_id.eq(device_id))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::referrer_username).aggregate_distinct())
            .first(&mut self.connection)
    }

    fn count_unique_referrers_for_fingerprint(&mut self, fingerprint: &str, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        dsl::rewards_risk_signals
            .filter(dsl::fingerprint.eq(fingerprint))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::referrer_username).aggregate_distinct())
            .first(&mut self.connection)
    }

    fn count_unique_devices_for_ip(&mut self, ip_address: &str, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        dsl::rewards_risk_signals
            .filter(dsl::ip_address.eq(ip_address))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::device_id).aggregate_distinct())
            .first(&mut self.connection)
    }

    fn count_unique_referrers_for_device_model_pattern(
        &mut self,
        device_model: &str,
        device_platform: &str,
        device_locale: &str,
        since: NaiveDateTime,
    ) -> Result<i64, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        dsl::rewards_risk_signals
            .filter(dsl::device_model.eq(device_model))
            .filter(dsl::device_platform.eq(device_platform))
            .filter(dsl::device_locale.eq(device_locale))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::referrer_username).aggregate_distinct())
            .first(&mut self.connection)
    }

    fn get_abuse_patterns_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<AbusePatterns, DieselError> {
        use crate::schema::rewards_risk_signals::dsl;

        let signals: Vec<RiskSignalRow> = dsl::rewards_risk_signals
            .filter(dsl::referrer_username.eq(referrer_username))
            .filter(dsl::created_at.ge(since))
            .select(RiskSignalRow::as_select())
            .load(&mut self.connection)?;

        if signals.is_empty() {
            return Ok(AbusePatterns::default());
        }

        let unique_devices: HashSet<i32> = signals.iter().map(|s| s.device_id).collect();
        let unique_fingerprints: HashSet<&str> = signals.iter().map(|s| s.fingerprint.as_str()).collect();
        let unique_ips: HashSet<&str> = signals.iter().map(|s| s.ip_address.as_str()).collect();

        let mut max_countries_per_device: i64 = 0;
        let mut max_referrers_per_device: i64 = 0;
        let mut max_referrers_per_fingerprint: i64 = 0;
        let mut max_devices_per_ip: i64 = 0;

        for device_id in unique_devices {
            let countries = self.count_unique_countries_for_device(device_id, since)?;
            max_countries_per_device = max_countries_per_device.max(countries);

            let referrers = self.count_unique_referrers_for_device(device_id, since)?;
            max_referrers_per_device = max_referrers_per_device.max(referrers);
        }

        for fingerprint in unique_fingerprints {
            let referrers = self.count_unique_referrers_for_fingerprint(fingerprint, since)?;
            max_referrers_per_fingerprint = max_referrers_per_fingerprint.max(referrers);
        }

        for ip_address in unique_ips {
            let devices = self.count_unique_devices_for_ip(ip_address, since)?;
            max_devices_per_ip = max_devices_per_ip.max(devices);
        }

        Ok(AbusePatterns {
            max_countries_per_device,
            max_referrers_per_device,
            max_referrers_per_fingerprint,
            max_devices_per_ip,
        })
    }
}
