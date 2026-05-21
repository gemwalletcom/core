use chrono::{DateTime, Utc};
use num_bigint::{BigInt, BigUint};
use primitives::{AssetId, Chain, NFTAssetId, NFTCollectionId, PerpetualId};
use std::str::FromStr;

uniffi::custom_type!(Chain, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| Chain::from_str(&s).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid Chain")),
});

uniffi::custom_type!(AssetId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| AssetId::new(&s).ok_or_else(|| uniffi::deps::anyhow::Error::msg("Invalid AssetId")),
});

uniffi::custom_type!(NFTAssetId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| NFTAssetId::from_str(&s).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid NFTAssetId")),
});

uniffi::custom_type!(NFTCollectionId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| NFTCollectionId::from_str(&s).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid NFTCollectionId")),
});

uniffi::custom_type!(PerpetualId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| PerpetualId::from_str(&s).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid PerpetualId")),
});

pub type GemBigInt = BigInt;
pub type GemBigUint = BigUint;

uniffi::custom_type!(GemBigInt, String, {
    remote,
    lower: |value| value.to_string(),
    try_lift: |s| BigInt::from_str(&s)
        .map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid BigInt")),
});

uniffi::custom_type!(GemBigUint, String, {
    remote,
    lower: |value| value.to_string(),
    try_lift: |s| BigUint::from_str(&s)
        .map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid BigUint")),
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
