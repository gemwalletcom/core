use chrono::{Duration, NaiveDateTime, Utc};
use std::time::Duration as StdDuration;

pub fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

pub trait DurationExt {
    fn as_days(&self) -> i64;
    fn as_days_ceil(&self) -> i64;
}

impl DurationExt for StdDuration {
    fn as_days(&self) -> i64 {
        (self.as_secs() / crate::duration::SECONDS_PER_DAY) as i64
    }

    fn as_days_ceil(&self) -> i64 {
        let seconds_days = self.as_secs().div_ceil(crate::duration::SECONDS_PER_DAY);
        let extra_day = u64::from(self.subsec_nanos() > 0 && self.as_secs().is_multiple_of(crate::duration::SECONDS_PER_DAY));
        (seconds_days + extra_day) as i64
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

    #[test]
    fn test_as_days_ceil() {
        assert_eq!(StdDuration::from_secs(0).as_days_ceil(), 0);
        assert_eq!(StdDuration::from_secs(1).as_days_ceil(), 1);
        assert_eq!(StdDuration::from_secs(12 * 60 * 60).as_days_ceil(), 1);
        assert_eq!(StdDuration::from_secs(36 * 60 * 60).as_days_ceil(), 2);
        assert_eq!(StdDuration::from_secs(7 * 24 * 60 * 60).as_days_ceil(), 7);
        assert_eq!(StdDuration::from_secs_f64(7.1 * 24.0 * 60.0 * 60.0).as_days_ceil(), 8);
    }
}
