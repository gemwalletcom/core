use primitives::{AssetId, Chain};
use std::str::FromStr;

uniffi::custom_type!(Chain, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| Chain::from_str(&s).map_err(|_| anyhow::anyhow!("Invalid Chain")),
});

uniffi::custom_type!(AssetId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| AssetId::new(&s).ok_or(anyhow::anyhow!("Invalid AssetId")),
});
