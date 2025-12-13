use chrono::{Duration, NaiveDateTime, Utc};

pub trait NaiveDateTimeExt {
    fn is_within_days(&self, days: i64) -> bool;
}

impl NaiveDateTimeExt for NaiveDateTime {
    fn is_within_days(&self, days: i64) -> bool {
        *self > Utc::now().naive_utc() - Duration::days(days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_within_days() {
        let now = Utc::now().naive_utc();
        assert!(now.is_within_days(1));
        assert!((now - Duration::days(6)).is_within_days(7));
        assert!(!(now - Duration::days(8)).is_within_days(7));
    }
}
