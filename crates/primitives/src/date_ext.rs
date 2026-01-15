use chrono::{Duration, NaiveDateTime, Utc};
use std::time::Duration as StdDuration;

pub fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

pub trait DurationExt {
    fn as_days(&self) -> i64;
}

impl DurationExt for StdDuration {
    fn as_days(&self) -> i64 {
        (self.as_secs() / 86400) as i64
    }
}

pub trait NaiveDateTimeExt {
    fn is_within_days(&self, days: i64) -> bool;
    fn is_older_than_days(&self, days: i64) -> bool;
    fn days_ago(&self, days: i64) -> NaiveDateTime;
    fn hours_ago(&self, hours: i64) -> NaiveDateTime;
    fn ago(&self, duration: StdDuration) -> NaiveDateTime;
    fn is_within_duration(&self, duration: StdDuration) -> bool;
}

impl NaiveDateTimeExt for NaiveDateTime {
    fn is_within_days(&self, days: i64) -> bool {
        *self > Utc::now().naive_utc() - Duration::days(days)
    }

    fn is_within_duration(&self, duration: StdDuration) -> bool {
        let chrono_duration = Duration::seconds(duration.as_secs() as i64);
        *self > Utc::now().naive_utc() - chrono_duration
    }

    fn is_older_than_days(&self, days: i64) -> bool {
        !self.is_within_days(days)
    }

    fn days_ago(&self, days: i64) -> NaiveDateTime {
        *self - Duration::days(days)
    }

    fn hours_ago(&self, hours: i64) -> NaiveDateTime {
        *self - Duration::hours(hours)
    }

    fn ago(&self, duration: StdDuration) -> NaiveDateTime {
        let chrono_duration = Duration::seconds(duration.as_secs() as i64);
        *self - chrono_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_within_days() {
        let now = Utc::now().naive_utc();
        assert!((now - Duration::days(6)).is_within_days(7));
        assert!(!(now - Duration::days(8)).is_within_days(7));
        assert!(!(now - Duration::days(7)).is_within_days(7));
    }

    #[test]
    fn test_is_older_than_days() {
        let now = Utc::now().naive_utc();
        assert!((now - Duration::days(8)).is_older_than_days(7));
        assert!(!(now - Duration::days(6)).is_older_than_days(7));
        assert!((now - Duration::days(7)).is_older_than_days(7));
    }
}
