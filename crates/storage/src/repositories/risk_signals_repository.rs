use crate::database::rewards::RiskSignalsStore;
use crate::models::{NewRiskSignalRow, RiskSignalRow};
use crate::{DatabaseClient, DatabaseError};
use chrono::NaiveDateTime;

pub trait RiskSignalsRepository {
    fn add_risk_signal(&mut self, signal: NewRiskSignalRow) -> Result<i32, DatabaseError>;
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
}

impl RiskSignalsRepository for DatabaseClient {
    fn add_risk_signal(&mut self, signal: NewRiskSignalRow) -> Result<i32, DatabaseError> {
        Ok(RiskSignalsStore::add_risk_signal(self, signal)?)
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
}
