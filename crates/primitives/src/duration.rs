use std::time::Duration;

pub fn parse_duration(raw: &str) -> Option<Duration> {
    let value = raw.trim();
    if value.is_empty() {
        return None;
    }

    if let Ok(seconds) = value.parse::<f64>() {
        return Some(Duration::from_secs_f64(seconds));
    }

    let split_index = value.find(|c: char| !c.is_ascii_digit() && c != '.')?;
    let (number, unit) = value.split_at(split_index);
    if number.is_empty() || unit.is_empty() || !unit.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }

    let amount = number.parse::<f64>().ok()?;
    match unit {
        "ns" => Some(Duration::from_nanos(amount as u64)),
        "us" => Some(Duration::from_micros(amount as u64)),
        "ms" => Some(Duration::from_millis(amount as u64)),
        "s" => Some(Duration::from_secs_f64(amount)),
        "m" => Some(Duration::from_secs_f64(amount * 60.0)),
        "h" => Some(Duration::from_secs_f64(amount * 3_600.0)),
        "d" => Some(Duration::from_secs_f64(amount * 86_400.0)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_seconds() {
        assert_eq!(parse_duration("3"), Some(Duration::from_secs(3)));
        assert_eq!(parse_duration("1.5"), Some(Duration::from_millis(1500)));
        assert_eq!(parse_duration("3s"), Some(Duration::from_secs(3)));
    }

    #[test]
    fn parse_units() {
        assert_eq!(parse_duration("1m"), Some(Duration::from_secs(60)));
        assert_eq!(parse_duration("1h"), Some(Duration::from_secs(3600)));
        assert_eq!(parse_duration("1d"), Some(Duration::from_secs(86400)));
        assert_eq!(parse_duration("500ms"), Some(Duration::from_millis(500)));
    }

    #[test]
    fn parse_invalid() {
        assert_eq!(parse_duration(""), None);
        assert_eq!(parse_duration("abc"), None);
        assert_eq!(parse_duration("1x"), None);
        assert_eq!(parse_duration("1s!"), None);
    }
}
