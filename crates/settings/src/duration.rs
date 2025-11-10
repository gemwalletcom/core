use serde::{Deserialize, Deserializer};
use std::time::Duration;

pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_duration(&s).map_err(serde::de::Error::custom)
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();

    if s.is_empty() {
        return Err("empty duration string".to_string());
    }

    let mut num_str = String::new();
    let mut unit = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() || c == '.' {
            num_str.push(c);
        } else if c.is_ascii_alphabetic() {
            unit.push(c);
        }
    }

    if num_str.is_empty() {
        return Err("no number found in duration".to_string());
    }

    let value: f64 = num_str.parse().map_err(|_| format!("invalid number: {}", num_str))?;

    let duration = match unit.as_str() {
        "ns" => Duration::from_nanos(value as u64),
        "us" => Duration::from_micros(value as u64),
        "ms" => Duration::from_millis(value as u64),
        "s" => Duration::from_secs_f64(value),
        "m" => Duration::from_secs_f64(value * 60.0),
        "h" => Duration::from_secs_f64(value * 3600.0),
        "d" => Duration::from_secs_f64(value * 86400.0),
        "" => Duration::from_millis(value as u64),
        _ => return Err(format!("unknown duration unit '{}', supported: ns, us, ms, s, m, h, d", unit)),
    };

    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("3s").unwrap(), Duration::from_secs(3));
        assert_eq!(parse_duration("3000ms").unwrap(), Duration::from_millis(3000));
        assert_eq!(parse_duration("1m").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("1.5s").unwrap(), Duration::from_millis(1500));
        assert_eq!(parse_duration("3000").unwrap(), Duration::from_millis(3000));
    }
}
