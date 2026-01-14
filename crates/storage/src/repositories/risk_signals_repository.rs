use crate::database::rewards::{AbusePatterns, RiskSignalsStore};
use crate::models::{NewRiskSignalRow, RiskSignalRow};
use crate::{DatabaseClient, DatabaseError};
use chrono::NaiveDateTime;

pub trait RiskSignalsRepository {
    fn add_risk_signal(&mut self, signal: NewRiskSignalRow) -> Result<i32, DatabaseError>;
    fn has_fingerprint_for_referrer(&mut self, fingerprint: &str, referrer_username: &str, since: NaiveDateTime) -> Result<bool, DatabaseError>;
    fn get_matching_risk_signals(
        &mut self,
        fingerprint: &str,
        ip_address: &str,
        ip_isp: &str,
        device_model: &str,
        device_id: i32,
        since: NaiveDateTime,
    ) -> Result<Vec<RiskSignalRow>, DatabaseError>;
    fn count_signals_since(&mut self, ip_address: Option<&str>, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_signals_for_device_id(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_signals_for_country(&mut self, country_code: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn sum_risk_scores_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_attempts_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn get_referrer_usernames_with_referrals(&mut self, since: NaiveDateTime, min_referrals: i64) -> Result<Vec<String>, DatabaseError>;
    fn count_unique_countries_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_unique_referrers_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_unique_referrers_for_fingerprint(&mut self, fingerprint: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_unique_devices_for_ip(&mut self, ip_address: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_unique_referrers_for_device_model_pattern(
        &mut self,
        device_model: &str,
        device_platform: &str,
        device_locale: &str,
        since: NaiveDateTime,
    ) -> Result<i64, DatabaseError>;
    fn get_abuse_patterns_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime, velocity_window_secs: i64) -> Result<AbusePatterns, DatabaseError>;
    fn count_disabled_users_by_ip(&mut self, ip_address: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
}

impl RiskSignalsRepository for DatabaseClient {
    fn add_risk_signal(&mut self, signal: NewRiskSignalRow) -> Result<i32, DatabaseError> {
        Ok(RiskSignalsStore::add_risk_signal(self, signal)?)
    }

    fn has_fingerprint_for_referrer(&mut self, fingerprint: &str, referrer_username: &str, since: NaiveDateTime) -> Result<bool, DatabaseError> {
        Ok(RiskSignalsStore::has_fingerprint_for_referrer(self, fingerprint, referrer_username, since)?)
    }

    fn get_matching_risk_signals(
        &mut self,
        fingerprint: &str,
        ip_address: &str,
        ip_isp: &str,
        device_model: &str,
        device_id: i32,
        since: NaiveDateTime,
    ) -> Result<Vec<RiskSignalRow>, DatabaseError> {
        Ok(RiskSignalsStore::get_matching_risk_signals(
            self,
            fingerprint,
            ip_address,
            ip_isp,
            device_model,
            device_id,
            since,
        )?)
    }

    fn count_signals_since(&mut self, ip_address: Option<&str>, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::count_signals_since(self, ip_address, since)?)
    }

    fn count_signals_for_device_id(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::count_signals_for_device_id(self, device_id, since)?)
    }

    fn count_signals_for_country(&mut self, country_code: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::count_signals_for_country(self, country_code, since)?)
    }

    fn sum_risk_scores_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::sum_risk_scores_for_referrer(self, referrer_username, since)?)
    }

    fn count_attempts_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::count_attempts_for_referrer(self, referrer_username, since)?)
    }

    fn get_referrer_usernames_with_referrals(&mut self, since: NaiveDateTime, min_referrals: i64) -> Result<Vec<String>, DatabaseError> {
        Ok(RiskSignalsStore::get_referrer_usernames_with_referrals(self, since, min_referrals)?)
    }

    fn count_unique_countries_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::count_unique_countries_for_device(self, device_id, since)?)
    }

    fn count_unique_referrers_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::count_unique_referrers_for_device(self, device_id, since)?)
    }

    fn count_unique_referrers_for_fingerprint(&mut self, fingerprint: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::count_unique_referrers_for_fingerprint(self, fingerprint, since)?)
    }

    fn count_unique_devices_for_ip(&mut self, ip_address: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::count_unique_devices_for_ip(self, ip_address, since)?)
    }

    fn count_unique_referrers_for_device_model_pattern(
        &mut self,
        device_model: &str,
        device_platform: &str,
        device_locale: &str,
        since: NaiveDateTime,
    ) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::count_unique_referrers_for_device_model_pattern(
            self,
            device_model,
            device_platform,
            device_locale,
            since,
        )?)
    }

    fn get_abuse_patterns_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime, velocity_window_secs: i64) -> Result<AbusePatterns, DatabaseError> {
        Ok(RiskSignalsStore::get_abuse_patterns_for_referrer(self, referrer_username, since, velocity_window_secs)?)
    }

    fn count_disabled_users_by_ip(&mut self, ip_address: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        Ok(RiskSignalsStore::count_disabled_users_by_ip(self, ip_address, since)?)
    }
}
