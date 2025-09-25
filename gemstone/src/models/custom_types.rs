use std::str::FromStr;

use chrono::{DateTime, Utc};
use num_bigint::{BigInt, BigUint};

uniffi::custom_type!(BigUint, String, {
    remote,
    lower: |value| value.to_string(),
    try_lift: |s| BigUint::from_str(&s)
        .map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid BigUint")),
});

uniffi::custom_type!(BigInt, String, {
    remote,
    lower: |value| value.to_string(),
    try_lift: |s| BigInt::from_str(&s)
        .map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid BigInt")),
});

pub type DateTimeUtc = DateTime<Utc>;

uniffi::custom_type!(DateTimeUtc, i64, {
    remote,
    lower: |value: DateTimeUtc| value.timestamp(),
    try_lift: |timestamp| {
        DateTime::<Utc>::from_timestamp(timestamp, 0)
            .ok_or_else(|| uniffi::deps::anyhow::Error::msg("Invalid timestamp"))
    },
});
