use chrono::{Duration, NaiveDateTime, Utc};

pub trait NaiveDateTimeExt {
    fn is_within_days(&self, days: i64) -> bool;
    fn is_older_than_days(&self, days: i64) -> bool;
}

impl NaiveDateTimeExt for NaiveDateTime {
    fn is_within_days(&self, days: i64) -> bool {
        *self > Utc::now().naive_utc() - Duration::days(days)
    }

    fn is_older_than_days(&self, days: i64) -> bool {
        !self.is_within_days(days)
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
